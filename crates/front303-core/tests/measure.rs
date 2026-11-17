//! Prints the models' pitch, level and cost: `cargo test --release --test measure -- --ignored --nocapture`.
mod common;
use common::*;
use front303_core::*;

#[test]
#[ignore]
fn measure() {
    for model in [Model::Rebirth, Model::RebirthVintage, Model::Dw303] {
        for (cut, res, env, acc) in [
            (0.16, 0.7, 0.16, 0.5),
            (0.5, 0.5, 0.5, 0.5),
            (0.2, 0.9, 0.8, 1.0),
            (1.0, 0.0, 0.0, 0.0),
        ] {
            let mut e = held(model, 0);
            e.patterns[0][0] = riff();
            let v = &mut e.params.voices[0];
            (v.cutoff, v.resonance, v.env_mod, v.accent, v.decay) = (cut, res, env, acc, 0.5);
            e.play();
            let t = std::time::Instant::now();
            let (l, _) = render(&mut e, 4.0);
            let cost = t.elapsed().as_secs_f64() / 4.0 * 100.0;
            println!(
                "{model:?} cutoff {cut} reso {res} env {env} accent {acc}: rms {:.1} dB, peak {:.1} dB, {cost:.1}% of real time",
                db(rms(&l)),
                db(peak(&l) as f64)
            );
        }
    }
}

/// Four voices of the circuit model at once, in both qualities.
#[test]
#[ignore]
fn four_circuit_voices() {
    for quality in [DwQuality::Full, DwQuality::Economy] {
        let mut e = Engine::new();
        e.params.dw_quality = quality;
        for c in 0..4 {
            e.patterns[c][0] = riff();
            e.params.voices[c].model = Model::Dw303;
            e.params.voices[c].cutoff = 0.5;
        }
        e.play();
        let t = std::time::Instant::now();
        render(&mut e, 8.0);
        println!(
            "4 x Dw303 {quality:?}: {:.0}% of real time",
            t.elapsed().as_secs_f64() / 8.0 * 100.0
        );
    }
}
