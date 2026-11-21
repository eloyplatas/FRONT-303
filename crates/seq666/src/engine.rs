//! The four sequencers, producing MIDI notes instead of sound.
//!
//! The timing is Front 303's: a position in sixteenth notes since the song's start, a step rate
//! per channel under the one tempo, shuffle on every second step, pattern changes latched at
//! the bar line, the evolve control rewriting a pattern as it comes round, quantisation to the
//! scale and MIDI transposition. Each step that sounds becomes a note-on at its exact sample and
//! a note-off after the channel's gate; a step that slides ties into the next one: the next
//! note starts before the held one ends (legato), which is how a synthesiser's portamento or a
//! TB-303 clone's slide is played from MIDI.

use front303_core::generator::{self, Rng};
use front303_core::params::{SLOTS, Scale};
use front303_core::pattern::{MAX_STEPS, Pattern, Step};

use crate::controls::{CHANNELS, SeqParams};

const BPM_MIN: f32 = 20.0;
const BPM_MAX: f32 = 300.0;
/// Steps per bar: a pattern change waits for the bar line.
const BAR: i64 = 16;
/// The most a shuffled step is delayed, as a share of a step.
const SHUFFLE_MAX: f64 = 0.5;
/// A host position this far from the sequencers', in sixteenths, moves them to it.
const SYNC_TOLERANCE: f64 = 0.05;
/// The pattern's C2, as a MIDI note.
const C2: i32 = 36;
pub const TRANSPOSE_RANGE: i32 = 24;
const SEED: u64 = 0x0666_0666;

/// One MIDI note event, at a sample of the block being processed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteEvent {
    pub timing: u32,
    pub on: bool,
    /// 0..15.
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

#[derive(Clone, Copy)]
struct Position {
    step: i64,
    gate_sent: bool,
}

impl Position {
    fn at(position: f64) -> Self {
        let step = position.floor();
        let within = position - step;
        if within < 1e-6 {
            Position {
                step: step as i64 - 1,
                gate_sent: false,
            }
        } else {
            Position {
                step: step as i64,
                gate_sent: true,
            }
        }
    }
}

pub struct Sequencer {
    pub params: SeqParams,
    pub patterns: [[Pattern; SLOTS]; CHANNELS],
    /// Counts the evolve control's rewrites per channel.
    pub evolutions: [u32; CHANNELS],
    /// Semitones the channels whose MIDI transpose is on are moved by.
    pub transpose: i32,
    active: [usize; CHANNELS],
    rng: Rng,
    playing: bool,
    position: f64,
    sixteenth: i64,
    positions: [Position; CHANNELS],
    stop_pending: bool,
    /// The note each channel holds: MIDI channel and note.
    held: [Option<(u8, u8)>; CHANNELS],
    /// The running step slides into the next.
    tie: [bool; CHANNELS],
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl Sequencer {
    pub fn new() -> Self {
        Sequencer {
            params: SeqParams::default(),
            patterns: [[Pattern::default(); SLOTS]; CHANNELS],
            evolutions: [0; CHANNELS],
            transpose: 0,
            active: [0; CHANNELS],
            rng: Rng::new(SEED),
            playing: false,
            position: 0.0,
            sixteenth: -1,
            positions: [Position {
                step: -1,
                gate_sent: false,
            }; CHANNELS],
            stop_pending: false,
            held: [None; CHANNELS],
            tie: [false; CHANNELS],
        }
    }

    pub fn play(&mut self) {
        self.play_from(0.0);
    }

    /// Starts at `position`, in sixteenth notes from the song's start.
    pub fn play_from(&mut self, position: f64) {
        self.playing = true;
        self.locate(position);
    }

    /// Moves to `position`; notes held are released.
    pub fn locate(&mut self, position: f64) {
        let position = if position.is_finite() { position } else { 0.0 };
        self.position = position;
        self.sixteenth = Position::at(position).step;
        for c in 0..CHANNELS {
            let rate = self.params.channels[c].rate.sixteenths();
            self.positions[c] = Position::at(position / rate);
        }
        self.stop_pending = true;
        self.latch_patterns();
    }

    /// Follows a host's transport at the start of a block.
    pub fn sync(&mut self, playing: bool, position: f64) {
        if !playing {
            self.stop();
        } else if !self.playing {
            self.play_from(position);
        } else if (self.position - position).abs() > SYNC_TOLERANCE {
            self.locate(position);
        }
    }

    pub fn stop(&mut self) {
        if self.playing {
            self.playing = false;
            self.stop_pending = true;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn tempo(&self) -> f64 {
        let b = self.params.bpm;
        (if b.is_finite() {
            b.clamp(BPM_MIN, BPM_MAX)
        } else {
            125.0
        }) as f64
    }

    fn latch_patterns(&mut self) {
        for (a, c) in self.active.iter_mut().zip(&self.params.channels) {
            *a = (c.pattern as usize).min(SLOTS - 1);
        }
    }

    pub fn active_pattern(&self, c: usize) -> usize {
        self.active[c]
    }

    /// The step of its pattern channel `c` is on, `None` when stopped.
    pub fn pattern_step(&self, c: usize) -> Option<usize> {
        let step = self.positions[c].step;
        if !self.playing || step < 0 {
            return None;
        }
        let len = (self.patterns[c][self.active[c]].len as i64).clamp(1, MAX_STEPS as i64);
        Some(step.rem_euclid(len) as usize)
    }

    /// Whether channel `c` is holding a note now.
    pub fn sounding(&self, c: usize) -> bool {
        self.held[c].is_some()
    }

    fn step_for(&mut self, c: usize, step: i64, first: bool) -> Step {
        let (root, scale) = (self.params.root, self.params.scale);
        let cp = self.params.channels[c];
        let g = cp.generator;
        let p = &mut self.patterns[c][self.active[c]];
        let len = (p.len as i64).clamp(1, MAX_STEPS as i64);
        if !first && step.rem_euclid(len) == 0 && g.evolve > 0.0 {
            generator::mutate(p, &g, g.evolve, root, scale, &mut self.rng);
            self.evolutions[c] = self.evolutions[c].wrapping_add(1);
        }
        let mut s = p.step(step);
        if scale != Scale::Chromatic {
            s.note = generator::quantize(s.note, root, scale);
        }
        s
    }

    fn release(&mut self, c: usize, timing: u32, emit: &mut impl FnMut(NoteEvent)) {
        if let Some((channel, note)) = self.held[c].take() {
            emit(NoteEvent {
                timing,
                on: false,
                channel,
                note,
                velocity: 0,
            });
        }
    }

    /// A step begins on channel `c`.
    fn start_step(&mut self, c: usize, s: Step, timing: u32, emit: &mut impl FnMut(NoteEvent)) {
        let cp = self.params.channels[c];
        let tied = self.tie[c] && self.held[c].is_some();
        if s.rest {
            // a tie into a rest carries the held note to the rest's gate-off
            if !tied {
                self.release(c, timing, emit);
            }
            self.tie[c] = false;
            return;
        }
        let transpose = if cp.midi_transpose {
            self.transpose.clamp(-TRANSPOSE_RANGE, TRANSPOSE_RANGE)
        } else {
            0
        };
        let note = (C2 + 12 * cp.octave as i32 + s.note as i32 + transpose).clamp(0, 127) as u8;
        let velocity = if s.accent {
            cp.accent_velocity
        } else {
            cp.velocity
        }
        .clamp(1, 127);
        let channel = cp.midi_channel.min(15);
        let next = (channel, note);
        if tied && self.held[c] == Some(next) {
            // a slide to the same note: the note goes on
        } else {
            if !tied {
                self.release(c, timing, emit);
            }
            emit(NoteEvent {
                timing,
                on: true,
                channel,
                note,
                velocity,
            });
            // legato: the new note starts before the old one ends
            if tied {
                self.release(c, timing, emit);
            }
            self.held[c] = Some(next);
        }
        self.tie[c] = s.slide;
    }

    /// Runs `samples` samples at `rate` Hz, giving every note event to `emit` in order.
    pub fn process(&mut self, samples: usize, rate: f64, mut emit: impl FnMut(NoteEvent)) {
        if core::mem::take(&mut self.stop_pending) {
            for c in 0..CHANNELS {
                self.release(c, 0, &mut emit);
                self.tie[c] = false;
            }
        }
        // a channel switched off lets go of its note
        for c in 0..CHANNELS {
            if !self.params.channels[c].on {
                self.release(c, 0, &mut emit);
                self.tie[c] = false;
            }
        }
        if !self.playing {
            self.latch_patterns();
            return;
        }
        let rate = if rate.is_finite() && rate > 1.0 {
            rate
        } else {
            44_100.0
        };
        let inc = self.tempo() * 4.0 / 60.0 / rate;
        let shuffle = self.params.shuffle;
        let late = if shuffle.is_finite() {
            shuffle.clamp(0.0, 1.0) as f64 * SHUFFLE_MAX
        } else {
            0.0
        };
        let rates = self.params.channels.map(|c| c.rate.sixteenths());
        let base = self.position;
        for i in 0..samples {
            let timing = i as u32;
            self.position = base + i as f64 * inc;
            let sixteenth = self.position.floor() as i64;
            if sixteenth != self.sixteenth {
                self.sixteenth = sixteenth;
                if sixteenth.rem_euclid(BAR) == 0 {
                    self.latch_patterns();
                }
            }
            for c in 0..CHANNELS {
                let position = self.position / rates[c];
                let mut step = position.floor() as i64;
                if step.rem_euclid(2) == 1 && position - (step as f64) < late {
                    step -= 1;
                }
                let start = step as f64 + if step.rem_euclid(2) == 1 { late } else { 0.0 };
                let pos = self.positions[c];
                if step > pos.step || step < pos.step - 1 {
                    let first = pos.step < 0;
                    self.positions[c] = Position {
                        step,
                        gate_sent: false,
                    };
                    let s = self.step_for(c, step, first);
                    if self.params.channels[c].on {
                        self.start_step(c, s, timing, &mut emit);
                    }
                }
                let gate = self.params.channels[c].gate.clamp(0.05, 1.0) as f64;
                if !self.positions[c].gate_sent && position - start >= gate.min(0.999) {
                    self.positions[c].gate_sent = true;
                    if !self.tie[c] {
                        self.release(c, timing, &mut emit);
                    }
                }
            }
        }
        self.position = base + samples as f64 * inc;
    }
}
