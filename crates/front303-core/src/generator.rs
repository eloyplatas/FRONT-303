//! The pattern generator: 303-style lines in a scale, and the edits that reshape one.
//!
//! Nothing here allocates, so the engine may call it from the audio thread (the evolve
//! control does) as well as a panel from its own.

use crate::params::{GenParams, Scale};
use crate::pattern::{MAX_STEPS, NOTE_MAX, NOTE_MIN, Pattern, Step};

/// A small deterministic generator (xorshift64*), so a seed gives the same patterns anywhere.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // any seed works, zero included
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in 0..1.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    /// Uniform in 0..n (0 when n is 0).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
}

fn unit(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The nearest note to `note` that is in the scale on `root` (a tie goes down), within the
/// pattern's range. Notes count semitones from a C, so `root` 0 is C.
pub fn quantize(note: i8, root: u8, scale: Scale) -> i8 {
    let intervals = scale.intervals();
    let root = (root % 12) as i32;
    let in_scale = |n: i32| intervals.contains(&((n - root).rem_euclid(12) as u8));
    let n = note as i32;
    for d in 0..12 {
        for candidate in [n - d, n + d] {
            if in_scale(candidate) && (NOTE_MIN as i32..=NOTE_MAX as i32).contains(&candidate) {
                return candidate as i8;
            }
        }
    }
    note
}

/// The register a generated line sits in: the root nearest below the keyboard's middle C,
/// which is where a 303 bass line lives.
fn base(root: u8) -> i32 {
    let root = (root % 12) as i32;
    if root <= 6 { root } else { root - 12 }
}

/// One note in the scale: mostly the root, often the fifth, sometimes the note before it
/// again, with octave jumps as `octaves` asks.
fn pick_note(g: &GenParams, root: u8, scale: Scale, previous: Option<i8>, rng: &mut Rng) -> i8 {
    if let Some(p) = previous
        && rng.chance(0.22)
    {
        return p;
    }
    let intervals = scale.intervals();
    let r = rng.unit();
    let degree = if r < 0.42 {
        0
    } else if r < 0.58 && intervals.contains(&7) {
        7
    } else {
        intervals[rng.below(intervals.len())] as i32
    };
    let mut note = base(root) + degree;
    let jump = rng.unit();
    let octaves = unit(g.octaves);
    if jump < octaves * 0.38 {
        note += 12;
    } else if jump < octaves * 0.58 {
        note -= 12;
    } else if jump < octaves * 0.64 {
        note += 24;
    }
    while note > NOTE_MAX as i32 {
        note -= 12;
    }
    while note < NOTE_MIN as i32 {
        note += 12;
    }
    note as i8
}

fn roll_step(
    index: usize,
    g: &GenParams,
    root: u8,
    scale: Scale,
    previous: Option<i8>,
    rng: &mut Rng,
) -> Step {
    let note = pick_note(g, root, scale, previous, rng);
    // the first step almost always plays; off-beats take the accents
    let density = unit(g.density);
    let play = if index == 0 {
        density.max(0.9)
    } else {
        density
    };
    let off_beat = index % 4 == 2 || index % 2 == 1;
    Step {
        note,
        rest: !rng.chance(play),
        accent: rng.chance(unit(g.accents) * if off_beat { 1.25 } else { 0.7 }),
        slide: rng.chance(unit(g.slides)),
    }
}

/// A slide ties a step into the next one, so it means something only on a played step that
/// a played step follows.
fn settle(p: &mut Pattern) {
    let len = (p.len as usize).clamp(1, MAX_STEPS);
    for i in 0..len {
        let next = p.steps[(i + 1) % len];
        let s = &mut p.steps[i];
        if s.rest {
            s.slide = false;
            s.accent = false;
        } else if next.rest {
            s.slide = false;
        }
    }
}

/// Writes a new line over the pattern's steps, keeping its length. A motif of half the
/// length is written first and repeated with changes, which is what makes a line a riff.
pub fn generate(p: &mut Pattern, g: &GenParams, root: u8, scale: Scale, rng: &mut Rng) {
    let len = (p.len as usize).clamp(1, MAX_STEPS);
    let motif = len.div_ceil(2).max(1);
    let mut previous = None;
    for i in 0..len {
        let fresh = i < motif || rng.chance(0.35);
        let step = if fresh {
            roll_step(i, g, root, scale, previous, rng)
        } else {
            p.steps[i - motif]
        };
        if !step.rest {
            previous = Some(step.note);
        }
        p.steps[i] = step;
    }
    if (0..len).all(|i| p.steps[i].rest) {
        p.steps[0].rest = false;
    }
    settle(p);
}

/// Rewrites about `amount` of the pattern's steps, one aspect of each: its note, its
/// accent, its slide, or whether it plays.
pub fn mutate(p: &mut Pattern, g: &GenParams, amount: f32, root: u8, scale: Scale, rng: &mut Rng) {
    let len = (p.len as usize).clamp(1, MAX_STEPS);
    let amount = unit(amount);
    for i in 0..len {
        if !rng.chance(amount) {
            continue;
        }
        let previous = (i > 0).then(|| p.steps[i - 1].note);
        let s = &mut p.steps[i];
        match rng.below(5) {
            0 | 1 => s.note = pick_note(g, root, scale, previous, rng),
            2 => s.accent = rng.chance(unit(g.accents)),
            3 => s.slide = rng.chance(unit(g.slides)),
            _ => s.rest = !rng.chance(unit(g.density)),
        }
    }
    if (0..len).all(|i| p.steps[i].rest) {
        p.steps[0].rest = false;
    }
    settle(p);
}

/// Moves every step `by` places later (earlier when negative), round the pattern's length.
pub fn rotate(p: &mut Pattern, by: i32) {
    let len = (p.len as usize).clamp(1, MAX_STEPS);
    let old = p.steps;
    for i in 0..len {
        p.steps[(i as i32 + by).rem_euclid(len as i32) as usize] = old[i];
    }
}

/// Moves every note by `semitones`, folding by octaves into the pattern's range.
pub fn transpose(p: &mut Pattern, semitones: i32) {
    for s in &mut p.steps {
        let mut n = s.note as i32 + semitones;
        while n > NOTE_MAX as i32 {
            n -= 12;
        }
        while n < NOTE_MIN as i32 {
            n += 12;
        }
        s.note = n as i8;
    }
}
