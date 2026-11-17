//! What the audio thread and a panel share, without a lock: the patterns, and what the panel
//! shows of the engine.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

use crate::params::{SLOTS, VOICES};
use crate::pattern::{MAX_STEPS, NOTE_MAX, NOTE_MIN, Pattern, Step};

fn pack(s: Step) -> u16 {
    let note = (s.note.clamp(NOTE_MIN, NOTE_MAX) - NOTE_MIN) as u16;
    note | (s.accent as u16) << 6 | (s.slide as u16) << 7 | (s.rest as u16) << 8
}

fn unpack(x: u16) -> Step {
    Step {
        note: ((x & 0x3F) as i8 + NOTE_MIN).clamp(NOTE_MIN, NOTE_MAX),
        accent: x & 0x40 != 0,
        slide: x & 0x80 != 0,
        rest: x & 0x100 != 0,
    }
}

/// Every voice's patterns. The panel edits them here; the audio thread copies them into the
/// engine when [`PatternBank::edits`] has moved, and writes back what the evolve control
/// changes.
pub struct PatternBank {
    steps: [[[AtomicU16; MAX_STEPS]; SLOTS]; VOICES],
    lens: [[AtomicU8; SLOTS]; VOICES],
    /// Counts the changes made outside the audio thread (the panel's, a state load).
    edits: AtomicU32,
    /// Counts every change, so a panel knows to redraw.
    version: AtomicU32,
}

impl Default for PatternBank {
    fn default() -> Self {
        let rest = pack(Step::default());
        PatternBank {
            steps: core::array::from_fn(|_| {
                core::array::from_fn(|_| core::array::from_fn(|_| AtomicU16::new(rest)))
            }),
            lens: core::array::from_fn(|_| {
                core::array::from_fn(|_| AtomicU8::new(MAX_STEPS as u8))
            }),
            edits: AtomicU32::new(0),
            version: AtomicU32::new(0),
        }
    }
}

impl PatternBank {
    pub fn get(&self, voice: usize, slot: usize) -> Pattern {
        let mut p = Pattern::default();
        for (s, a) in p.steps.iter_mut().zip(&self.steps[voice][slot]) {
            *s = unpack(a.load(Relaxed));
        }
        p.len = self.lens[voice][slot]
            .load(Relaxed)
            .clamp(1, MAX_STEPS as u8);
        p
    }

    fn store(&self, voice: usize, slot: usize, p: &Pattern) {
        for (s, a) in p.steps.iter().zip(&self.steps[voice][slot]) {
            a.store(pack(*s), Relaxed);
        }
        self.lens[voice][slot].store(p.len.clamp(1, MAX_STEPS as u8), Relaxed);
        self.version.fetch_add(1, Relaxed);
    }

    /// A change from the panel or a state load: the audio thread picks it up.
    pub fn set(&self, voice: usize, slot: usize, p: &Pattern) {
        self.store(voice, slot, p);
        self.edits.fetch_add(1, Relaxed);
    }

    /// A change the audio thread made itself.
    pub fn set_from_audio(&self, voice: usize, slot: usize, p: &Pattern) {
        self.store(voice, slot, p);
    }

    pub fn edits(&self) -> u32 {
        self.edits.load(Relaxed)
    }

    pub fn version(&self) -> u32 {
        self.version.load(Relaxed)
    }

    /// Every pattern as text: per pattern its length and sixteen steps, in hexadecimal.
    pub fn serialize(&self) -> String {
        let mut out = String::with_capacity(VOICES * SLOTS * (2 + MAX_STEPS * 3));
        for v in 0..VOICES {
            for s in 0..SLOTS {
                let p = self.get(v, s);
                out.push_str(&format!("{:02x}", p.len));
                for step in p.steps {
                    out.push_str(&format!("{:03x}", pack(step)));
                }
            }
        }
        out
    }

    /// Reads [`PatternBank::serialize`]'s text. Text of another length or with other
    /// characters changes nothing and returns false.
    pub fn deserialize(&self, text: &str) -> bool {
        let per = 2 + MAX_STEPS * 3;
        if text.len() != VOICES * SLOTS * per || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return false;
        }
        for v in 0..VOICES {
            for s in 0..SLOTS {
                let chunk = &text[(v * SLOTS + s) * per..][..per];
                let mut p = Pattern::default();
                p.len = u8::from_str_radix(&chunk[..2], 16)
                    .unwrap_or(16)
                    .clamp(1, MAX_STEPS as u8);
                for (i, step) in p.steps.iter_mut().enumerate() {
                    let x = u16::from_str_radix(&chunk[2 + i * 3..][..3], 16).unwrap_or(0x100);
                    *step = unpack(x);
                }
                self.store(v, s, &p);
            }
        }
        self.edits.fetch_add(1, Relaxed);
        true
    }
}

/// What the panel shows of the running engine.
pub struct Feedback {
    pub playing: AtomicBool,
    /// The host's transport is driving the patterns.
    pub host: AtomicBool,
    /// The step each voice is on in its pattern, -1 when stopped.
    pub steps: [AtomicI32; VOICES],
    /// The pattern each voice is playing.
    pub active: [AtomicU8; VOICES],
    /// The channels' peaks (f32 bits), then the master's left and right, then the effect
    /// returns'.
    peaks: [AtomicU32; VOICES + 3],
    /// The MIDI transposition in semitones.
    pub transpose: AtomicI32,
    /// The tempo in use, BPM (f32 bits).
    tempo: AtomicU32,
}

impl Default for Feedback {
    fn default() -> Self {
        Feedback {
            playing: AtomicBool::new(false),
            host: AtomicBool::new(false),
            steps: core::array::from_fn(|_| AtomicI32::new(-1)),
            active: core::array::from_fn(|_| AtomicU8::new(0)),
            peaks: core::array::from_fn(|_| AtomicU32::new(0)),
            transpose: AtomicI32::new(0),
            tempo: AtomicU32::new(125f32.to_bits()),
        }
    }
}

impl Feedback {
    /// Raises peak `i` (a channel, `VOICES` and `VOICES + 1` for the master, `VOICES + 2` for
    /// the effect returns) to `x`.
    pub fn raise_peak(&self, i: usize, x: f32) {
        if x > f32::from_bits(self.peaks[i].load(Relaxed)) {
            self.peaks[i].store(x.to_bits(), Relaxed);
        }
    }

    /// Reads peak `i` and clears it, so the next reading is the peak since this one.
    pub fn take_peak(&self, i: usize) -> f32 {
        f32::from_bits(self.peaks[i].swap(0, Relaxed))
    }

    pub fn set_tempo(&self, bpm: f32) {
        self.tempo.store(bpm.to_bits(), Relaxed);
    }

    pub fn tempo(&self) -> f32 {
        f32::from_bits(self.tempo.load(Relaxed))
    }
}
