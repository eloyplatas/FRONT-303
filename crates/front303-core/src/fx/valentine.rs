//! Valentine (Tote Bag Labs, GPL-3.0): input gain into a feed-forward peak compressor whose
//! knee and threshold follow the ratio, then an asymmetric inverse-hyperbolic-sine saturator
//! and a soft clip, all at 4× the rate; optional "crush" (a zero-order hold towards 27.5 kHz
//! and a bit crusher) before it; makeup gain and a dry/wet mix after.

use super::db;
use super::os::Os4;
use crate::SAMPLE_RATE;
use crate::params::ValentineParams;

const OVERSAMPLING: f64 = 4.0;
/// Valentine's detector time constant.
const TIME_CONSTANT: f64 = -0.99967234;
const RATIO_POINTS: [f64; 6] = [1.0, 4.0, 8.0, 12.0, 20.0, 1000.0];
const KNEE_POINTS: [f64; 6] = [7.0, 6.0, 3.84, 2.16, 0.96, 0.0];
const THRESHOLD_POINTS: [f64; 6] = [-15.0, -18.0, -14.0, -13.0, -12.0, -10.0];
const ASYMMETRY: f64 = 0.6;
const SATURATION_MIN: f64 = 0.5;
const SATURATION_MAX: f64 = 10.0;
/// The output clip's gain (−6 dB).
const CLIP_GAIN: f64 = 0.501_187_233_6;
const DOWNSAMPLE_RATE: f64 = 27_500.0;

fn remap(points_in: &[f64; 6], points_out: &[f64; 6], x: f64) -> f64 {
    for i in 0..5 {
        if x <= points_in[i + 1] {
            let t = (x - points_in[i]) / (points_in[i + 1] - points_in[i]);
            return points_out[i] + t.clamp(0.0, 1.0) * (points_out[i + 1] - points_out[i]);
        }
    }
    points_out[5]
}

/// Attack 0..127 over 0.02..55 ms, release over 2..1100 ms, both exponential.
pub(crate) fn attack_ms(b: u8) -> f64 {
    0.02 * (55.0f64 / 0.02).powf(b.min(127) as f64 / 127.0)
}

pub(crate) fn release_ms(b: u8) -> f64 {
    2.0 * (1100.0f64 / 2.0).powf(b.min(127) as f64 / 127.0)
}

/// COMPRESS from the AMOUNT byte: −6..+36 dB.
pub(crate) fn compress_db(b: u8) -> f64 {
    -6.0 + 42.0 * b.min(127) as f64 / 127.0
}

/// OUTPUT from the THRESHOLD byte: −24..+12 dB.
pub(crate) fn output_db(b: u8) -> f64 {
    -24.0 + 36.0 * b.min(127) as f64 / 127.0
}

fn gain_to_db(g: f64) -> f64 {
    if g > 0.0 {
        (20.0 * g.log10()).max(-100.0)
    } else {
        -100.0
    }
}

/// First-order antiderivative anti-aliasing for one channel.
#[derive(Clone, Copy, Default)]
struct Adaa {
    x: f64,
    ad: f64,
}

impl Adaa {
    fn asinh(&mut self, x: f64) -> f64 {
        let ad = x * x.asinh() - (x * x + 1.0).sqrt();
        let diff = x - self.x;
        let y = if diff.abs() < 1.0e-4 {
            ((x + self.x) / 2.0).asinh()
        } else {
            (ad - self.ad) / diff
        };
        self.x = x;
        self.ad = ad;
        y
    }

    fn tanh(&mut self, x: f64) -> f64 {
        let ad = x.clamp(-700.0, 700.0).cosh().ln();
        let diff = x - self.x;
        let y = if diff.abs() < 1.0e-6 {
            ((x + self.x) / 2.0).tanh()
        } else {
            (ad - self.ad) / diff
        };
        self.x = x;
        self.ad = ad;
        y
    }
}

fn asinh_compensation(g: f64) -> f64 {
    if g <= 1.02 { 1.0 / g } else { 1.0 / g.asinh() }
}

fn tanh_compensation(g: f64) -> f64 {
    if g <= 1.02 { 1.0 / g } else { 1.0 / g.tanh() }
}

pub(crate) struct Valentine {
    os: [Os4; 2],
    envelope: f64,
    saturate: [Adaa; 2],
    clip: [Adaa; 2],
    input_gain: f64,
    makeup: f64,
    mix: f64,
    /// The last gain reduction, as a gain.
    pub(crate) reduction: f64,
}

impl Valentine {
    pub(crate) fn new() -> Self {
        Valentine {
            os: [Os4::new(), Os4::new()],
            envelope: 0.0,
            saturate: [Adaa::default(); 2],
            clip: [Adaa::default(); 2],
            input_gain: 1.0,
            makeup: 1.0,
            mix: 1.0,
            reduction: 1.0,
        }
    }

    pub(crate) fn reset(&mut self) {
        for o in &mut self.os {
            o.reset();
        }
        self.envelope = 0.0;
        self.saturate = [Adaa::default(); 2];
        self.clip = [Adaa::default(); 2];
    }

    fn crush(x: &mut [f32], bits: f64) {
        // Valentine's zero-order hold works in place within a block, at 44.1/27.5 = 1.6
        let coefficient = SAMPLE_RATE / DOWNSAMPLE_RATE;
        let whole = coefficient as usize;
        let frac = coefficient - whole as f64;
        let hold = |x: &[f32], i: usize, d: usize| -> f64 {
            if d <= 1 {
                x[i] as f64
            } else {
                x[i - i % d] as f64
            }
        };
        for i in 0..x.len() {
            let y1 = hold(x, i, whole);
            let y2 = hold(x, i, whole + 1);
            x[i] = (y1 + frac * (y2 - y1)) as f32;
        }
        let q = 1.0 / 2f64.powf(bits);
        for s in x.iter_mut() {
            *s = (q * (*s as f64 / q).floor()) as f32;
        }
    }

    /// Processes one block: one channel, or two linked by a mono sidechain.
    pub(crate) fn process(
        &mut self,
        p: &ValentineParams,
        compress: f64,
        output: f64,
        left: &mut [f32],
        mut right: Option<&mut [f32]>,
    ) {
        let n = left.len();
        let channels = if right.is_some() { 2 } else { 1 };
        let mut dry = [[0.0f32; 64]; 2];
        dry[0][..n].copy_from_slice(left);
        if let Some(r) = right.as_deref() {
            dry[1][..n].copy_from_slice(r);
        }
        if p.crush > 0 {
            // CRUSH 1..10 sets 12..1 bits
            let bits = 12.0 - 11.0 * (p.crush as f64 - 1.0) / 9.0;
            Self::crush(left, bits);
            if let Some(r) = right.as_deref_mut() {
                Self::crush(r, bits);
            }
        }
        let ratio = (p.ratio as f64).clamp(1.0, 21.0);
        let knee = remap(&RATIO_POINTS, &KNEE_POINTS, ratio);
        let threshold = remap(&RATIO_POINTS, &THRESHOLD_POINTS, ratio);
        let rate = SAMPLE_RATE * OVERSAMPLING;
        let attack = (TIME_CONSTANT / (0.001 * attack_ms(p.attack) * rate)).exp();
        let release = (TIME_CONSTANT / (0.001 * release_ms(p.release) * rate)).exp();
        let saturation = (p.saturate > 0).then(|| {
            SATURATION_MIN + (SATURATION_MAX - SATURATION_MIN) * (p.saturate as f64 - 1.0) / 9.0
        });
        let (g0, g1) = (self.input_gain, db(compress));
        let (m0, m1) = (self.makeup, db(output));
        let (x0, x1) = (self.mix, p.mix.min(100) as f64 / 100.0);
        self.input_gain = g1;
        self.makeup = m1;
        self.mix = x1;
        let mut least = 1.0f64;
        for i in 0..n {
            let t = (i + 1) as f64 / n as f64;
            let gain = g0 + (g1 - g0) * t;
            let mut up = [[0.0f64; 4]; 2];
            up[0] = self.os[0].up(left[i] as f64 * gain);
            if let Some(r) = right.as_deref() {
                up[1] = self.os[1].up(r[i] as f64 * gain);
            }
            for k in 0..4 {
                let sidechain = (0..channels).map(|c| up[c][k]).sum::<f64>() / channels as f64;
                let level = gain_to_db(sidechain.abs());
                let coefficient = if level > self.envelope {
                    attack
                } else {
                    release
                };
                self.envelope = coefficient * (self.envelope - level) + level;
                let reduction = db(gain_computer(self.envelope, threshold, knee, ratio));
                least = least.min(reduction);
                for c in 0..channels {
                    let mut y = up[c][k] * reduction;
                    if let Some(sat) = saturation {
                        let g = if y >= 0.0 {
                            sat * (1.0 + 4.0 * ASYMMETRY)
                        } else {
                            sat
                        };
                        y = self.saturate[c].asinh(y * g) * asinh_compensation(g);
                    }
                    y = self.clip[c].tanh(y * CLIP_GAIN) * tanh_compensation(CLIP_GAIN);
                    up[c][k] = y;
                }
            }
            let makeup = m0 + (m1 - m0) * t;
            let mix = x0 + (x1 - x0) * t;
            let y = self.os[0].down(up[0]) * makeup;
            left[i] = (mix * y + (1.0 - mix) * dry[0][i] as f64) as f32;
            if let Some(r) = right.as_deref_mut() {
                let y = self.os[1].down(up[1]) * makeup;
                r[i] = (mix * y + (1.0 - mix) * dry[1][i] as f64) as f32;
            }
        }
        self.reduction = least;
    }
}

/// Valentine's soft-knee gain computer: the gain change in dB for a detector level in dB.
fn gain_computer(level: f64, threshold: f64, knee: f64, ratio: f64) -> f64 {
    let overshoot = level - threshold;
    let double = 2.0 * overshoot;
    let out = if knee > 0.0 {
        if double < -knee {
            level
        } else if double.abs() <= knee {
            level + (1.0 / ratio - 1.0) * (overshoot + knee / 2.0).powi(2) / (2.0 * knee)
        } else {
            threshold + overshoot / ratio
        }
    } else if level > threshold {
        threshold + overshoot / ratio
    } else {
        level
    };
    -(level - out)
}
