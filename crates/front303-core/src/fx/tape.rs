//! IV-X Delay's tape echo (Joaquim Broquetas Ruiz, GPL-3.0), run at 4× the engine's rate.
//!
//! Each sample, the transport's delay time creeps towards its target (the pitch bend of a
//! tape machine changing speed); tape noise and the feedback join the input; the record head
//! is a measured transfer curve and FIR response for the chosen tape speed and bias; the
//! playback head reads with linear interpolation at a position swayed by wow, flutter and
//! scrape. The curves and responses are IV-X's own tables. IV-X's recorded noise tables are
//! not distributed with its source, so the noise here is synthesised: white noise shaped
//! darker for the slower speeds.
//!
//! IV-X runs at 192 kHz; here the rate is 176.4 kHz, with the modulation depths and the
//! transport's inertia rescaled to keep their timings. The FIR responses scale with the rate,
//! by the 8 % between the two.

use std::sync::OnceLock;

use super::Noise;
use super::os::Os4;
use crate::SAMPLE_RATE;

const OVERSAMPLING: usize = 4;
const RATE: f64 = SAMPLE_RATE * OVERSAMPLING as f64;
const ORIGINAL_RATE: f64 = 192_000.0;
/// The longest echo, in seconds.
pub(crate) const MAX_TIME: f64 = 2.5;
const SHAPER_SIZE: usize = 32768;
const FIR_SIZE: usize = 128;
const OSC_SIZE: usize = 4096;
const FEEDBACK_TRIM: f64 = 0.7;
/// Input level into the record head per speed (fast, medium, slow).
const IPS_LEVEL: [f64; 3] = [0.4, 0.55, 0.8];
const IPS_NOISE: [f64; 3] = [0.006, 0.005, 0.012];
/// The synthesised noise's low-pass corner per speed.
const NOISE_CORNER: [f64; 3] = [12_000.0, 7_000.0, 3_500.0];
const WOW_RATE: f64 = 1.0;
const FLUTTER_RATE: f64 = 15.0;
const SCRAPE_RATE: f64 = 100.0;

struct Tables {
    shaper: Vec<f32>,
    fir: Vec<f32>,
    sine: Vec<f32>,
    scrape: Vec<f32>,
}

fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        use core::f64::consts::TAU;
        // IV-X's scrape oscillator: 32 harmonics with fixed phases
        const HARMONICS: [(f64, f64); 32] = [
            (0.025, 0.67),
            (0.0289, 0.42),
            (0.0313, 0.86),
            (0.0156, 0.38),
            (0.0165, 0.71),
            (0.0264, 0.29),
            (0.0198, 0.92),
            (0.0249, 0.54),
            (0.0195, 0.13),
            (0.0158, 0.66),
            (0.0213, 0.45),
            (0.0259, 0.88),
            (0.0176, 0.31),
            (0.0165, 0.65),
            (0.0129, 0.32),
            (0.0224, 0.76),
            (0.0164, 0.25),
            (0.0218, 0.12),
            (0.0089, 0.84),
            (0.0123, 0.48),
            (0.0074, 0.64),
            (0.0189, 0.86),
            (0.0220, 0.34),
            (0.0103, 0.75),
            (0.0152, 0.62),
            (0.0086, 0.54),
            (0.0120, 0.92),
            (0.0089, 0.53),
            (0.0104, 0.81),
            (0.0120, 0.10),
            (0.0056, 0.08),
            (0.0216, 0.0),
        ];
        let sine = (0..OSC_SIZE)
            .map(|i| (TAU * i as f64 / OSC_SIZE as f64).sin() as f32)
            .collect();
        let scrape = (0..OSC_SIZE)
            .map(|i| {
                let x = i as f64 / OSC_SIZE as f64;
                let mut v = 0.0;
                for (k, &(a, phase)) in HARMONICS.iter().enumerate() {
                    // the 32nd entry is IV-X's fundamental
                    let h = if k == 31 { 1.0 } else { k as f64 + 2.0 };
                    v += a * (TAU * h * x + phase * TAU).sin();
                }
                v as f32
            })
            .collect();
        Tables {
            shaper: floats(include_bytes!("../../assets/ivx_shaper.bin")),
            fir: floats(include_bytes!("../../assets/ivx_fir.bin")),
            sine,
            scrape,
        }
    })
}

/// Linear interpolation in a periodic table at a phase in table units.
fn lookup(table: &[f32], phase: f64) -> f64 {
    let i = phase as usize;
    let frac = phase - i as f64;
    let a = table[i % OSC_SIZE] as f64;
    let b = table[(i + 1) % OSC_SIZE] as f64;
    a + frac * (b - a)
}

/// The panel of the tape echo, in IV-X's units.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TapeSettings {
    /// Seconds.
    pub(crate) time: f64,
    /// 0..1.5.
    pub(crate) feedback: f64,
    pub(crate) wow: f64,
    pub(crate) flutter: f64,
    pub(crate) scrape: f64,
    /// 0.01..4.
    pub(crate) noise: f64,
    /// 0 fast (15 ips), 1 medium (7.5), 2 slow (3.75).
    pub(crate) speed: usize,
    /// 0..3 for IV-X's bias 2..5.
    pub(crate) bias: usize,
}

pub(crate) struct Tape {
    buffer: Vec<f32>,
    write: usize,
    time: f64,
    fir: [f64; FIR_SIZE],
    fir_pos: usize,
    feedback_sample: f64,
    wow: f64,
    flutter: f64,
    scrape: f64,
    noise: Noise,
    noise_lp: f64,
    noise_sample: f64,
    os: Os4,
}

impl Tape {
    pub(crate) fn new() -> Self {
        tables();
        Tape {
            buffer: vec![0.0; (MAX_TIME * RATE) as usize + 4096],
            write: 0,
            time: 0.0,
            fir: [0.0; FIR_SIZE],
            fir_pos: 0,
            feedback_sample: 0.0,
            wow: 0.0,
            flutter: 0.0,
            scrape: 0.0,
            noise: Noise::new(0x1717),
            noise_lp: 0.0,
            noise_sample: 0.0,
            os: Os4::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.fir = [0.0; FIR_SIZE];
        self.feedback_sample = 0.0;
        self.os.reset();
    }

    /// The input's level that the record head turns into its output's (a calibration aid).
    pub(crate) fn input_level(speed: usize) -> f64 {
        IPS_LEVEL[speed.min(2)]
    }

    /// Mono in, mono out (the echo only).
    pub(crate) fn process(&mut self, s: &TapeSettings, input: &[f32], output: &mut [f32]) {
        let t = tables();
        let speed = s.speed.min(2);
        let table = speed * 4 + s.bias.min(3);
        let shaper = &t.shaper[table * SHAPER_SIZE..(table + 1) * SHAPER_SIZE];
        let fir = &t.fir[table * FIR_SIZE..(table + 1) * FIR_SIZE];
        let level = IPS_LEVEL[speed];
        let noise_level = s.noise * IPS_NOISE[speed];
        let noise_k = 1.0 - (-core::f64::consts::TAU * NOISE_CORNER[speed] / RATE).exp();
        let target = s.time.clamp(0.001, MAX_TIME - 0.01);
        // IV-X's transport creeps 1e-5 of the way per sample at 192 kHz
        let creep = 1.0 - (1.0 - 1.0e-5f64).powf(ORIGINAL_RATE / RATE);
        let depth = RATE / ORIGINAL_RATE;
        let feedback = s.feedback.clamp(0.0, 1.5) * FEEDBACK_TRIM;
        let osc = OSC_SIZE as f64 / RATE;
        let size = self.buffer.len();
        for (x, y) in input.iter().zip(output.iter_mut()) {
            let up = self.os.up(*x as f64);
            let mut out = [0.0; OVERSAMPLING];
            for (k, &u) in up.iter().enumerate() {
                self.time += creep * (target - self.time);
                let offset = self.time * RATE;
                // noise: white, darkened per speed
                self.noise_lp += noise_k * (self.noise.next() - self.noise_lp);
                self.noise_sample = self.noise_lp * 2.0;
                let mut v = u + noise_level * self.noise_sample + feedback * self.feedback_sample;
                // record head: transfer curve, then the FIR response
                v *= level;
                v = if v >= 1.0 {
                    1.0
                } else if v <= -1.0 {
                    -1.0
                } else {
                    let pos = 0.5 * SHAPER_SIZE as f64 * (1.0 + v);
                    let i = (pos as usize).min(SHAPER_SIZE - 2);
                    let a = shaper[i] as f64;
                    a + (pos - i as f64) * (shaper[i + 1] as f64 - a)
                };
                self.fir[self.fir_pos] = v;
                let mut acc = 0.0;
                let mut r = self.fir_pos;
                for &c in fir {
                    acc += self.fir[r] * c as f64;
                    r = if r == 0 { FIR_SIZE - 1 } else { r - 1 };
                }
                self.fir_pos = (self.fir_pos + 1) % FIR_SIZE;
                self.buffer[self.write] = acc as f32;
                self.write = (self.write + 1) % size;
                // playback head
                self.wow = (self.wow + WOW_RATE * osc) % OSC_SIZE as f64;
                self.flutter = (self.flutter + FLUTTER_RATE * osc) % OSC_SIZE as f64;
                self.scrape = (self.scrape + SCRAPE_RATE * osc) % OSC_SIZE as f64;
                let sway = 1500.0 * s.wow * lookup(&t.sine, self.wow)
                    + 50.0 * s.flutter * lookup(&t.sine, self.flutter)
                    + 35.0 * s.scrape * (lookup(&t.scrape, self.scrape) + 0.6 * self.noise_sample);
                let mut read = self.write as f64 - offset + sway * depth;
                while read < 0.0 {
                    read += size as f64;
                }
                while read >= size as f64 {
                    read -= size as f64;
                }
                let i = read as usize;
                let a = self.buffer[i] as f64;
                let b = self.buffer[(i + 1) % size] as f64;
                let played = a + (read - i as f64) * (b - a);
                self.feedback_sample = played;
                out[k] = played;
            }
            *y = self.os.down(out) as f32;
        }
    }
}
