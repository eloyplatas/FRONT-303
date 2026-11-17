//! What the audio thread and a panel share, without a lock: the patterns, and what the panel
//! shows of the engine.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

use crate::params::{SLOTS, VOICES};
use crate::pattern::{MAX_STEPS, Pattern, Step};
use crate::snapshot::{pack, patterns_from_hex, patterns_to_hex, unpack};

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

    /// Every pattern, voice by voice.
    pub fn all(&self) -> Vec<Pattern> {
        let mut out = Vec::with_capacity(VOICES * SLOTS);
        for v in 0..VOICES {
            for s in 0..SLOTS {
                out.push(self.get(v, s));
            }
        }
        out
    }

    /// Every pattern as text: per pattern its length and sixteen steps, in hexadecimal.
    pub fn serialize(&self) -> String {
        patterns_to_hex(&self.all())
    }

    /// Reads [`PatternBank::serialize`]'s text. Text of another length or with other
    /// characters changes nothing and returns false.
    pub fn deserialize(&self, text: &str) -> bool {
        let Some(patterns) = patterns_from_hex(text, VOICES * SLOTS) else {
            return false;
        };
        for v in 0..VOICES {
            for s in 0..SLOTS {
                self.store(v, s, &patterns[v * SLOTS + s]);
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
