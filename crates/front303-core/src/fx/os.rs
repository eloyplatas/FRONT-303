//! A 4× oversampler: two half-band IIR stages each way.

use ravet_signal::{ConfiguredHalfBand2x, HalfBandIirDesign};

pub(crate) struct Os4 {
    first: ConfiguredHalfBand2x,
    second: ConfiguredHalfBand2x,
}

impl Os4 {
    pub(crate) fn new() -> Self {
        let design =
            HalfBandIirDesign::from_attenuation(0.1, 90.0).expect("valid half-band design");
        Os4 {
            first: ConfiguredHalfBand2x::new(design, design),
            second: ConfiguredHalfBand2x::new(design, design),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.first.reset();
        self.second.reset();
    }

    pub(crate) fn up(&mut self, x: f64) -> [f64; 4] {
        let [a, b] = self.first.upsample(x);
        let [c, d] = self.second.upsample(a);
        let [e, f] = self.second.upsample(b);
        [c, d, e, f]
    }

    pub(crate) fn down(&mut self, x: [f64; 4]) -> f64 {
        let a = self.second.downsample([x[0], x[1]]);
        let b = self.second.downsample([x[2], x[3]]);
        self.first.downsample([a, b])
    }
}
