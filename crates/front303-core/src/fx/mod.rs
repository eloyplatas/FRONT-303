//! The effects Front 303 adds to ReBirth's: the Setekh distortion, the Valentine compressor,
//! the IV-X tape delay, the interval delay and the reverbs.

pub(crate) mod biquad;
pub mod intervals;
pub(crate) mod os;
pub mod reverb;
pub(crate) mod setekh;
pub(crate) mod tape;
pub(crate) mod valentine;

/// dB to gain.
pub(crate) fn db(x: f64) -> f64 {
    10f64.powf(x / 20.0)
}

/// A small, fast noise source (xorshift32), uniform in −1..1.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Noise(u32);

impl Noise {
    pub(crate) fn new(seed: u32) -> Self {
        Noise(seed.max(1))
    }

    pub(crate) fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}
