//! A delay whose repeats sit on a pattern of intervals, after Matthew Fecher's Delay Intervals
//! (MIT): the input runs down one line; at every multiple of the interval it passes a tap,
//! where the falloff gain and that tap's filters are applied to the line itself, so each
//! repeat is darker and quieter than the one before; each tap is heard at its own left and
//! right amplitude. The last tap may loop back into the line's input.
//!
//! The original sets the sixteen amplitudes per side by hand; here they come from a set of
//! patterns, and one TONE control sets the filters.

use super::biquad::Biquad;
use crate::SAMPLE_RATE;

/// Taps at most.
pub(crate) const MAX_TAPS: usize = 16;
/// The longest interval, in seconds.
pub(crate) const MAX_INTERVAL: f64 = 1.0;

/// A pattern: its number of taps and each tap's amplitude, left and right.
pub(crate) struct IntervalPattern {
    pub(crate) name: &'static str,
    pub(crate) taps: usize,
    pub(crate) left: [f32; MAX_TAPS],
    pub(crate) right: [f32; MAX_TAPS],
}

const fn pattern(
    name: &'static str,
    taps: usize,
    left: [f32; MAX_TAPS],
    right: [f32; MAX_TAPS],
) -> IntervalPattern {
    IntervalPattern {
        name,
        taps,
        left,
        right,
    }
}

const O: f32 = 0.0;
const H: f32 = 0.5;
const I: f32 = 1.0;

/// Tap 0 is the input itself; the repeats are taps 1.. .
pub(crate) const PATTERNS: [IntervalPattern; 8] = [
    pattern(
        "STRAIGHT",
        8,
        [O, I, I, I, I, I, I, I, O, O, O, O, O, O, O, O],
        [O, I, I, I, I, I, I, I, O, O, O, O, O, O, O, O],
    ),
    pattern(
        "PING PONG",
        12,
        [O, I, O, I, O, I, O, I, O, I, O, I, O, O, O, O],
        [O, O, I, O, I, O, I, O, I, O, I, O, O, O, O, O],
    ),
    pattern(
        "DOTTED",
        13,
        [O, O, O, I, O, O, I, O, O, I, O, O, I, O, O, O],
        [O, O, O, I, O, O, I, O, O, I, O, O, I, O, O, O],
    ),
    pattern(
        "TRESILLO",
        16,
        [O, O, O, I, O, O, I, O, O, O, O, I, O, O, I, O],
        [O, O, O, H, O, O, I, O, I, O, O, H, O, O, I, I],
    ),
    pattern(
        "SWELL",
        12,
        [
            O, 0.15, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.0, O, O, O, O,
        ],
        [
            O, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.0, O, O, O, O,
        ],
    ),
    pattern(
        "GALLOP",
        12,
        [O, I, I, O, I, I, O, I, I, O, I, I, O, O, O, O],
        [O, H, I, O, H, I, O, H, I, O, H, I, O, O, O, O],
    ),
    pattern(
        "SCATTER",
        16,
        [O, I, O, O, H, O, I, I, O, O, O, H, O, I, O, O],
        [O, O, H, I, O, O, O, I, O, I, H, O, O, O, I, H],
    ),
    pattern(
        "STUTTER",
        16,
        [O, I, I, I, I, O, O, O, O, O, O, O, I, I, I, I],
        [O, I, I, I, I, O, O, O, O, O, O, O, I, I, I, I],
    ),
];

/// The name of interval pattern `i` (0..8).
pub fn pattern_name(i: usize) -> &'static str {
    PATTERNS[i.min(PATTERNS.len() - 1)].name
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IntervalSettings {
    /// Seconds per interval.
    pub(crate) interval: f64,
    pub(crate) pattern: usize,
    /// 0..1: the share of level lost at each tap.
    pub(crate) falloff: f64,
    /// 0 dark, 0.5 open, 1 thin.
    pub(crate) tone: f64,
    /// 0..1: how much of the last tap loops back in.
    pub(crate) loop_gain: f64,
    /// 0 both sides alike (the left pattern), 1 the pattern's own left and right.
    pub(crate) spread: f64,
}

pub(crate) struct Intervals {
    line: Vec<f32>,
    write: usize,
    filters: [[Biquad; 2]; MAX_TAPS],
    tone: f64,
    interval: usize,
}

impl Intervals {
    pub(crate) fn new() -> Self {
        let size = ((MAX_INTERVAL * SAMPLE_RATE) as usize + 1) * (MAX_TAPS + 1);
        let mut s = Intervals {
            line: vec![0.0; size],
            write: 0,
            filters: [[Biquad::default(); 2]; MAX_TAPS],
            tone: -1.0,
            interval: 0,
        };
        s.set_tone(0.5);
        s
    }

    pub(crate) fn reset(&mut self) {
        self.line.fill(0.0);
        for f in self.filters.iter_mut().flatten() {
            f.reset();
        }
    }

    fn set_tone(&mut self, tone: f64) {
        if tone == self.tone {
            return;
        }
        self.tone = tone;
        // below the centre a low-pass closes to 1 kHz, above it a high-pass opens to 1.5 kHz
        let lp = if tone < 0.5 {
            1000.0 * 20f64.powf(tone * 2.0)
        } else {
            20_000.0
        };
        let hp = if tone > 0.5 {
            20.0 * 75f64.powf((tone - 0.5) * 2.0)
        } else {
            20.0
        };
        for f in &mut self.filters {
            f[0].high_pass(SAMPLE_RATE, hp);
            f[1].low_pass(SAMPLE_RATE, lp);
        }
    }

    /// Mono in; the repeats are added to `left` and `right`.
    pub(crate) fn process(
        &mut self,
        s: &IntervalSettings,
        input: &[f32],
        left: &mut [f32],
        right: &mut [f32],
    ) {
        let p = &PATTERNS[s.pattern.min(PATTERNS.len() - 1)];
        let interval = ((s.interval.clamp(0.005, MAX_INTERVAL) * SAMPLE_RATE) as usize).max(1);
        if interval != self.interval {
            // a new interval scrambles the line: start it clean, as the original fades it out
            if self.interval != 0 {
                self.line.fill(0.0);
            }
            self.interval = interval;
        }
        self.set_tone(s.tone.clamp(0.0, 1.0));
        let keep = 1.0 - s.falloff.clamp(0.0, 1.0);
        let spread = s.spread.clamp(0.0, 1.0) as f32;
        let size = self.line.len();
        let taps = p.taps.min(MAX_TAPS);
        for i in 0..input.len() {
            // the loop: the line's last tap, through tap 0's filters, back into its input
            let mut x = input[i] as f64;
            if s.loop_gain > 0.0 {
                let at = (self.write + size - interval * taps) % size;
                let v = self.line[at] as f64 * keep;
                let [hp, lp] = &mut self.filters[0];
                let v = lp.tick(hp.tick(v));
                self.line[at] = v as f32;
                x += v * s.loop_gain.min(1.0);
            }
            self.line[self.write] = x as f32;
            let (mut l, mut r) = (0.0f32, 0.0f32);
            for tap in (1..taps).rev() {
                let at = (self.write + size - interval * tap) % size;
                let v = self.line[at] as f64 * keep;
                let [hp, lp] = &mut self.filters[tap];
                let v = lp.tick(hp.tick(v)) as f32;
                self.line[at] = v;
                let gl = p.left[tap];
                let gr = p.left[tap] + spread * (p.right[tap] - p.left[tap]);
                l += gl * v;
                r += gr * v;
            }
            left[i] += l;
            right[i] += r;
            self.write = (self.write + 1) % size;
        }
    }
}
