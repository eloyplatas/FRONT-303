//! The engine runs at 44.1 kHz; this brings its output to the host's rate.
//!
//! A windowed-sinc interpolator: 32 taps of a Kaiser-windowed sinc, tabulated at 1024
//! phases and interpolated linearly between them. The cutoff sits below 20 kHz when the host
//! runs faster than the engine, and below the host's Nyquist frequency when it runs slower.
//! Nothing allocates after [`Resampler::new`].

use front303_core::SAMPLE_RATE;

/// The most channels a resampler carries.
pub const CHANNELS: usize = front303_core::OUTPUTS;

const HALF: usize = 16;
const TAPS: usize = 2 * HALF;
const PHASES: usize = 1024;
const BETA: f64 = 8.0;
/// The engine samples the interpolator keeps: a power of two above `TAPS` plus a block.
const RING: usize = 256;

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let mut k = 1.0;
    while term > 1e-12 * sum {
        term *= (x / (2.0 * k)) * (x / (2.0 * k));
        sum += term;
        k += 1.0;
    }
    sum
}

pub struct Resampler {
    /// Engine samples per host sample.
    step: f64,
    /// `None` when the host runs at the engine's rate.
    table: Option<Vec<[f32; TAPS]>>,
    ring: [[f32; RING]; CHANNELS],
    /// Engine samples written into the ring, in all.
    written: u64,
    /// The position of the next output, in engine samples (its integer part is the newest
    /// sample the kernel's centre has passed).
    pos: f64,
}

impl Resampler {
    pub fn new(host_rate: f64) -> Self {
        let host_rate = if host_rate.is_finite() && host_rate > 0.0 {
            host_rate
        } else {
            SAMPLE_RATE
        };
        let step = SAMPLE_RATE / host_rate;
        let table = ((host_rate - SAMPLE_RATE).abs() > 0.5).then(|| {
            // the cutoff as a share of the engine's Nyquist frequency
            let cutoff = if host_rate > SAMPLE_RATE {
                19_500.0 / (SAMPLE_RATE / 2.0)
            } else {
                0.91 * host_rate / SAMPLE_RATE
            };
            let norm = bessel_i0(BETA);
            (0..=PHASES)
                .map(|p| {
                    let frac = p as f64 / PHASES as f64;
                    let mut taps = [0f32; TAPS];
                    let mut sum = 0.0;
                    let mut raw = [0f64; TAPS];
                    for (k, r) in raw.iter_mut().enumerate() {
                        // tap k is at engine sample (base - HALF + 1 + k); the output at base + frac
                        let x = k as f64 - (HALF as f64 - 1.0) - frac;
                        let t = x / HALF as f64;
                        let window = if t.abs() >= 1.0 {
                            0.0
                        } else {
                            bessel_i0(BETA * (1.0 - t * t).sqrt()) / norm
                        };
                        let arg = std::f64::consts::PI * cutoff * x;
                        let sinc = if x.abs() < 1e-12 {
                            1.0
                        } else {
                            arg.sin() / arg
                        };
                        *r = cutoff * sinc * window;
                        sum += *r;
                    }
                    for (t, r) in taps.iter_mut().zip(raw) {
                        *t = (r / sum) as f32;
                    }
                    taps
                })
                .collect()
        });
        Resampler {
            step,
            table,
            ring: [[0.0; RING]; CHANNELS],
            written: 0,
            pos: (HALF - 1) as f64,
        }
    }

    /// The delay the interpolator adds, in host samples: none. The engine is a generator, so
    /// the samples the kernel needs ahead of the output are simply rendered sooner; the output
    /// starts `HALF - 1` engine samples (0.34 ms) into the engine's time, which
    /// [`Resampler::buffered`] accounts for.
    pub fn latency(&self) -> u32 {
        0
    }

    /// Engine samples rendered but not yet heard, so a caller can tell where the engine's
    /// position is relative to what plays now.
    pub fn buffered(&self) -> f64 {
        if self.table.is_some() {
            self.written as f64 - self.pos
        } else {
            0.0
        }
    }

    /// Fills `left` and `right` (the same length), asking `render` for engine samples.
    /// Fills `outputs` (at most [`CHANNELS`], all the same length) at the host's rate;
    /// `render` fills as many channels at 44.1 kHz.
    pub fn process(
        &mut self,
        outputs: &mut [&mut [f32]],
        mut render: impl FnMut(&mut [&mut [f32]]),
    ) {
        let Some(table) = &self.table else {
            render(outputs);
            return;
        };
        let channels = outputs.len().min(CHANNELS);
        let len = outputs.first().map_or(0, |o| o.len());
        let mut chunks = [[0f32; 64]; CHANNELS];
        for n in 0..len {
            let base = self.pos.floor() as u64;
            while self.written < base + HALF as u64 + 1 {
                {
                    let mut refs = chunks.each_mut().map(|c| &mut c[..]);
                    render(&mut refs[..channels]);
                }
                for c in 0..channels {
                    for i in 0..64 {
                        let at = (self.written as usize + i) & (RING - 1);
                        self.ring[c][at] = chunks[c][i];
                    }
                }
                self.written += 64;
            }
            let frac = (self.pos - base as f64) * PHASES as f64;
            let p = (frac as usize).min(PHASES - 1);
            let mix = (frac - p as f64) as f32;
            let (a, b) = (&table[p], &table[p + 1]);
            let first = base + 1 - HALF as u64;
            let mut w = [0f32; TAPS];
            for k in 0..TAPS {
                w[k] = a[k] + (b[k] - a[k]) * mix;
            }
            for c in 0..channels {
                let ring = &self.ring[c];
                let mut y = 0f32;
                for k in 0..TAPS {
                    y += ring[(first as usize + k) & (RING - 1)] * w[k];
                }
                outputs[c][n] = y;
            }
            self.pos += self.step;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(host: f64, hz: f64) -> Vec<f32> {
        let mut rs = Resampler::new(host);
        let mut n = 0u64;
        let mut out_l = vec![0.0; host as usize];
        let mut out_r = vec![0.0; host as usize];
        for (l, r) in out_l.chunks_mut(441).zip(out_r.chunks_mut(441)) {
            rs.process(&mut [l, r], |out| {
                let [a, b, ..] = out else { return };
                for (x, y) in a.iter_mut().zip(b.iter_mut()) {
                    *x = (2.0 * std::f64::consts::PI * hz * n as f64 / SAMPLE_RATE).sin() as f32;
                    *y = *x;
                    n += 1;
                }
            });
        }
        out_l
    }

    /// The tone's amplitude at `hz` and everything else's, measured against a reference sine.
    fn fit(x: &[f32], host: f64, hz: f64, delay: f64) -> (f64, f64) {
        let x = &x[1000..];
        let w = 2.0 * std::f64::consts::PI * hz / host;
        let (mut err, mut sig) = (0.0, 0.0);
        for (i, &v) in x.iter().enumerate() {
            let t = (i + 1000) as f64 - delay;
            let reference = (w * t).sin();
            sig += reference * reference;
            err += (v as f64 - reference) * (v as f64 - reference);
        }
        (sig, err)
    }

    #[test]
    fn a_sine_survives_at_common_rates() {
        for host in [48_000.0, 88_200.0, 96_000.0, 192_000.0, 32_000.0] {
            for hz in [55.0, 1000.0, 9000.0] {
                let rs = Resampler::new(host);
                // the output runs HALF - 1 engine samples ahead of the engine's start
                let delay = -((HALF as f64 - 1.0) * host / SAMPLE_RATE);
                let x = tone(host, hz);
                let (sig, err) = fit(&x, host, hz, delay);
                let snr = 10.0 * (sig / err).log10();
                assert!(snr > 60.0, "{host} Hz host, {hz} Hz tone: {snr:.1} dB");
                assert_eq!(rs.latency(), 0);
            }
        }
    }

    #[test]
    fn the_engine_rate_passes_unchanged() {
        let mut rs = Resampler::new(SAMPLE_RATE);
        let (mut l, mut r) = ([0.0f32; 100], [0.0f32; 100]);
        rs.process(&mut [&mut l[..], &mut r[..]], |out| {
            out[0].fill(0.5);
            out[1].fill(-0.5);
        });
        assert!(l.iter().all(|&x| x == 0.5) && r.iter().all(|&x| x == -0.5));
        assert_eq!(rs.latency(), 0);
    }
}
