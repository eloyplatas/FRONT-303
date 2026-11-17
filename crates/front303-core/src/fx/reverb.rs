//! The reverb: four engines (Dragonfly Hall, Dragonfly Early Reflections, Aether and
//! CloudReverb), all built when the engine is, of which only the chosen one runs. Front 303
//! adds a predelay in front and a mid/side width control after.

use front303_cxx::{REVERB_BLOCK, Reverb as Cxx, ReverbKind};

use crate::SAMPLE_RATE;
use crate::params::{ReverbMode, ReverbParams};

/// The longest predelay, in seconds.
const MAX_PREDELAY: f64 = 0.25;

/// Per-program trims (dB) that bring the programs' impulse energies near one another.
const TRIMS: [[f32; 8]; 4] = [
    [-2.0, 1.7, -0.8, 0.0, -0.8, 0.3, -0.7, 0.3],
    [-2.9, -3.7, 0.2, -3.7, -0.7, -2.2, -2.0, -1.8],
    [-0.6, 7.0, -0.4, -1.4, 8.0, -7.3, 2.2, -3.2],
    [8.0, 6.4, 8.0, 8.0, 4.5, 6.8, 4.4, 4.5],
];

fn kind(mode: ReverbMode) -> ReverbKind {
    match mode {
        ReverbMode::Hall => ReverbKind::Hall,
        ReverbMode::Early => ReverbKind::Early,
        ReverbMode::Aether => ReverbKind::Aether,
        ReverbMode::Cloud => ReverbKind::Cloud,
    }
}

/// A program's name.
pub fn program_name(mode: ReverbMode, program: usize) -> &'static str {
    kind(mode).program_name(program)
}

pub(crate) struct ReverbUnit {
    engines: [Cxx; 4],
    current: usize,
    predelay: [Vec<f32>; 2],
    write: usize,
    trim: f32,
}

impl ReverbUnit {
    pub(crate) fn new() -> Self {
        let size = (MAX_PREDELAY * SAMPLE_RATE) as usize + REVERB_BLOCK + 1;
        ReverbUnit {
            engines: ReverbMode::ALL.map(|m| Cxx::new(kind(m), SAMPLE_RATE)),
            current: 0,
            predelay: [vec![0.0; size], vec![0.0; size]],
            write: 0,
            trim: 1.0,
        }
    }

    pub(crate) fn reset(&mut self) {
        for e in &mut self.engines {
            e.clear();
        }
        for p in &mut self.predelay {
            p.fill(0.0);
        }
    }

    /// One block (at most 64 samples): `input` is the stereo send, the wet signal is added to
    /// `output`.
    pub(crate) fn process(
        &mut self,
        p: &ReverbParams,
        input: [&[f32]; 2],
        output: [&mut [f32]; 2],
    ) {
        let n = input[0].len().min(REVERB_BLOCK);
        let mode = p.mode as usize;
        if mode != self.current {
            // the engine taking over starts from silence
            self.engines[mode].clear();
            self.current = mode;
        }
        let program = (p.program as usize).min(7);
        let engine = &mut self.engines[mode];
        engine.program(program);
        engine.shape(p.size, p.decay, p.damping, p.modulation);
        let size = self.predelay[0].len();
        let delay = ((p.predelay.clamp(0.0, 1.0) as f64 * MAX_PREDELAY * SAMPLE_RATE) as usize)
            .min(size - REVERB_BLOCK - 1);
        let mut delayed = [[0.0f32; REVERB_BLOCK]; 2];
        for i in 0..n {
            for c in 0..2 {
                self.predelay[c][self.write] = input[c][i];
                delayed[c][i] = self.predelay[c][(self.write + size - delay) % size];
            }
            self.write = (self.write + 1) % size;
        }
        let mut wet = [[0.0f32; REVERB_BLOCK]; 2];
        {
            let [wl, wr] = &mut wet;
            engine.process(
                &delayed[0][..n],
                &delayed[1][..n],
                &mut wl[..n],
                &mut wr[..n],
            );
        }
        let target = 10f32.powf(TRIMS[mode][program] / 20.0) * p.level.clamp(0.0, 1.0);
        let width = p.width.clamp(0.0, 1.0) * 1.5;
        let [ol, or] = output;
        let start = self.trim;
        for i in 0..n {
            let g = start + (target - start) * (i + 1) as f32 / n as f32;
            let (l, r) = (wet[0][i], wet[1][i]);
            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r) * width;
            if !(mid.is_finite() && side.is_finite()) {
                continue;
            }
            ol[i] += g * (mid + side);
            or[i] += g * (mid - side);
        }
        self.trim = target;
    }
}
