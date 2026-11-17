//! The Open303 voice: Robin Schmidt's Open303 (MIT; maddanio's fork), compiled from its C++
//! source, played by Front 303's sequencer.
//!
//! The knobs follow Open303's own plug-in: cutoff 314–2394 Hz and decay 200–2000 ms on
//! exponential tapers, resonance, env mod and accent 0–100 %. Tune and fine set the pitch
//! directly: the note's MIDI number takes the semitones, the reference A4 the cents.

use front303_cxx::{Open303, Open303Panel};
use orb_engine::frame::Block;

use crate::SAMPLE_RATE;
use crate::params::{VoiceParams, Wave, knob, tune};
use crate::pattern::{Event, Events};

/// The MIDI note of the sequencer's C2.
const C2: i32 = 36;
/// Open303 is hot (accented notes peak well above full scale); this brings a riff's level to
/// the ReBirth voice's.
const LEVEL_TRIM: f32 = 0.49;

pub(crate) struct O3Voice {
    synth: Open303,
    note: i32,
    gate: bool,
    hold: bool,
}

impl O3Voice {
    pub(crate) fn new() -> Self {
        O3Voice {
            synth: Open303::new(SAMPLE_RATE),
            note: 0,
            gate: false,
            hold: false,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.synth.silence();
        self.gate = false;
        self.hold = false;
    }

    fn midi(&self, p: &VoiceParams) -> i32 {
        let (semis, _) = tune(p);
        (C2 + self.note + semis).clamp(0, 127)
    }

    fn event(&mut self, event: Event, p: &VoiceParams) {
        match event {
            Event::Step(s) => {
                // the step before this one tied into it
                let tied = self.hold && self.gate;
                self.note = s.note as i32;
                if s.rest {
                    // as on the other voices, a tie into a rest carries its pitch and ends at
                    // the rest's gate-off
                    if tied {
                        let n = self.midi(p);
                        self.synth.slide(n, false);
                    }
                    self.hold = false;
                    return;
                }
                let n = self.midi(p);
                if tied {
                    self.synth.slide(n, s.accent);
                } else {
                    self.synth.trigger(n, s.accent);
                }
                self.gate = true;
                self.hold = s.slide;
            }
            Event::GateOff => {
                if !self.hold && self.gate {
                    self.synth.release();
                    self.gate = false;
                }
            }
            Event::AllOff => {
                self.synth.silence();
                self.hold = false;
                self.gate = false;
            }
        }
    }

    pub(crate) fn render(&mut self, bus: &mut Block, events: &Events, p: &VoiceParams) {
        let (_, cents) = tune(p);
        self.synth.panel(&Open303Panel {
            cutoff_hz: 314.0 * (2394.0f64 / 314.0).powf(knob(p.cutoff)),
            resonance: 100.0 * knob(p.resonance),
            env_mod: 100.0 * knob(p.env_mod),
            decay_ms: 200.0 * 10f64.powf(knob(p.decay)),
            accent: 100.0 * knob(p.accent),
            waveform: if p.wave == Wave::Square { 1.0 } else { 0.0 },
            tuning_hz: 440.0 * 2f64.powf(cents / 1200.0),
        });
        let mut start = 0;
        for (offset, e) in events.iter() {
            let offset = offset.min(bus.len());
            if offset > start {
                self.synth.render(&mut bus[start..offset]);
                start = offset;
            }
            self.event(e, p);
        }
        if start < bus.len() {
            self.synth.render(&mut bus[start..]);
        }
        for x in bus.iter_mut() {
            *x *= LEVEL_TRIM;
        }
    }
}
