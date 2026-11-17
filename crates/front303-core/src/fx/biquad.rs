//! Second-order filters with JUCE's coefficient formulas, which the ported effects use.

use core::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Biquad {
    b: [f64; 3],
    a: [f64; 2],
    z: [f64; 2],
}

impl Biquad {
    fn set(&mut self, b: [f64; 3], a: [f64; 3]) {
        let inv = 1.0 / a[0];
        self.b = [b[0] * inv, b[1] * inv, b[2] * inv];
        self.a = [a[1] * inv, a[2] * inv];
    }

    /// `juce::dsp::IIR::Coefficients::makeHighShelf`.
    pub(crate) fn high_shelf(&mut self, rate: f64, freq: f64, q: f64, gain: f64) {
        let a = gain.max(0.0).sqrt();
        let (am, ap) = (a - 1.0, a + 1.0);
        let omega = 2.0 * PI * freq.max(2.0) / rate;
        let coso = omega.cos();
        let beta = omega.sin() * a.sqrt() / q;
        let amc = am * coso;
        self.set(
            [
                a * (ap + amc + beta),
                a * -2.0 * (am + ap * coso),
                a * (ap + amc - beta),
            ],
            [ap - amc + beta, 2.0 * (am - ap * coso), ap - amc - beta],
        );
    }

    /// `makeLowPass` with a Q of 1/√2.
    pub(crate) fn low_pass(&mut self, rate: f64, freq: f64) {
        let n = 1.0 / (PI * freq.clamp(10.0, 0.49 * rate) / rate).tan();
        let n2 = n * n;
        let iq = 2f64.sqrt();
        let c1 = 1.0 / (1.0 + iq * n + n2);
        self.set(
            [c1, c1 * 2.0, c1],
            [1.0, c1 * 2.0 * (1.0 - n2), c1 * (1.0 - iq * n + n2)],
        );
    }

    /// `makeHighPass` with a Q of 1/√2.
    pub(crate) fn high_pass(&mut self, rate: f64, freq: f64) {
        let n = (PI * freq.clamp(5.0, 0.49 * rate) / rate).tan();
        let n2 = n * n;
        let iq = 2f64.sqrt();
        let c1 = 1.0 / (1.0 + iq * n + n2);
        self.set(
            [c1, c1 * -2.0, c1],
            [1.0, c1 * 2.0 * (n2 - 1.0), c1 * (1.0 - iq * n + n2)],
        );
    }

    pub(crate) fn reset(&mut self) {
        self.z = [0.0; 2];
    }

    /// Transposed direct form II.
    #[inline]
    pub(crate) fn tick(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.z[0];
        self.z[0] = self.b[1] * x - self.a[0] * y + self.z[1];
        self.z[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}
