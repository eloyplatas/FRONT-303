//! Setekh (Full FX Media, GPL-3.0): a +6 dB high shelf at 7.5 kHz, input gain, a blend of the
//! clean signal and tanh(3.5·x) by the drive, output gain, and the shelf taken back out.
//!
//! The original runs one shelf filter across both channels; here every channel has its own.

use super::biquad::Biquad;
use super::db;
use crate::SAMPLE_RATE;

const SHELF_HZ: f64 = 7500.0;
const SHELF_Q: f64 = 0.707;

#[derive(Clone, Copy)]
pub(crate) struct Setekh {
    boost: Biquad,
    cut: Biquad,
}

impl Setekh {
    pub(crate) fn new() -> Self {
        let mut s = Setekh {
            boost: Biquad::default(),
            cut: Biquad::default(),
        };
        s.boost.high_shelf(SAMPLE_RATE, SHELF_HZ, SHELF_Q, db(6.0));
        s.cut.high_shelf(SAMPLE_RATE, SHELF_HZ, SHELF_Q, db(-6.0));
        s
    }

    pub(crate) fn reset(&mut self) {
        self.boost.reset();
        self.cut.reset();
    }

    /// `drive` 0..5, gains in dB (−24..24).
    pub(crate) fn process(&mut self, x: &mut [f32], drive: f64, input_db: f64, output_db: f64) {
        let input = db(input_db);
        let output = db(output_db);
        let amount = (drive / 5.0).clamp(0.0, 1.0);
        for s in x.iter_mut() {
            let clean = self.boost.tick(*s as f64) * input;
            let distorted = (clean * 5.0 * 0.7).tanh();
            let wet = (1.0 - amount) * clean + amount * distorted;
            *s = self.cut.tick(wet * output) as f32;
        }
    }
}
