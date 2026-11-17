//! Helpers shared by the integration tests.
#![allow(dead_code)]

use front303_core::{Engine, Model, Pattern, SAMPLE_RATE, Step};

/// An engine whose voice 0 holds `note` for ever (one tied step), alone and centred.
pub fn held(model: Model, note: i8) -> Engine {
    let mut e = Engine::new();
    for c in 1..4 {
        e.params.channels[c].on = false;
    }
    e.params.voices[0].model = model;
    e.patterns[0][0] = Pattern::from_steps(&[Step::note(note).slide()]);
    e
}

pub fn render(e: &mut Engine, seconds: f64) -> (Vec<f32>, Vec<f32>) {
    let n = (seconds * SAMPLE_RATE) as usize;
    let mut l = vec![0.0; n];
    let mut r = vec![0.0; n];
    // odd chunk sizes, as a host would ask
    for (a, b) in l.chunks_mut(441).zip(r.chunks_mut(441)) {
        e.render(a, b);
    }
    (l, r)
}

pub fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&v| v as f64 * v as f64).sum::<f64>() / x.len() as f64).sqrt()
}

pub fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |m, &v| m.max(v.abs()))
}

pub fn db(x: f64) -> f64 {
    20.0 * x.max(1e-12).log10()
}

/// The fundamental of a steady tone near `expected` Hz: the mean spacing of the rising zero
/// crossings of the signal, low-passed at `expected` so the fundamental dominates.
pub fn frequency(x: &[f32], expected: f64) -> f64 {
    let k = 1.0 - (-2.0 * std::f64::consts::PI * expected / SAMPLE_RATE).exp();
    let mean = x.iter().map(|&v| v as f64).sum::<f64>() / x.len() as f64;
    let (mut a, mut b, mut c) = (0.0, 0.0, 0.0);
    let mut y = Vec::with_capacity(x.len());
    for &v in x {
        a += (v as f64 - mean - a) * k;
        b += (a - b) * k;
        c += (b - c) * k;
        y.push(c);
    }
    let y = &y[y.len() / 4..];
    let mean = y.iter().sum::<f64>() / y.len() as f64;
    let mut crossings = Vec::new();
    for i in 1..y.len() {
        let (p, q) = (y[i - 1] - mean, y[i] - mean);
        if p < 0.0 && q >= 0.0 {
            crossings.push(i as f64 - 1.0 + p / (p - q));
        }
    }
    assert!(crossings.len() > 4, "no steady tone");
    let periods = (crossings.len() - 1) as f64;
    SAMPLE_RATE * periods / (crossings[crossings.len() - 1] - crossings[0])
}

pub fn cents(f: f64, reference: f64) -> f64 {
    1200.0 * (f / reference).log2()
}

/// A sixteen-step line with accents, slides and octave jumps.
pub fn riff() -> Pattern {
    let n = Step::note;
    Pattern::from_steps(&[
        n(0),
        n(0).accent(),
        n(12),
        n(0),
        n(3).slide(),
        n(5),
        n(0).accent(),
        n(-12),
        n(0),
        n(7).slide(),
        n(10).accent(),
        n(0),
        n(12),
        n(0).accent(),
        n(3),
        n(5),
    ])
}

pub const MODELS: [Model; 4] = [
    Model::Rebirth,
    Model::RebirthVintage,
    Model::Dw303,
    Model::Open303,
];
