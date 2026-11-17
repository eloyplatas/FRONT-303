//! The voices: tuning, the tune and fine controls, levels, and the range's ends.
mod common;
use common::*;
use front303_core::*;

/// The steady pitch of voice 0 holding `note` at the given tuning.
fn pitch(model: Model, note: i8, tune: i8, fine: f32, expected: f64) -> f64 {
    let mut e = held(model, note);
    let v = &mut e.params.voices[0];
    v.tune = tune;
    v.fine = fine;
    v.cutoff = 0.8;
    v.resonance = 0.0;
    e.play();
    let (l, _) = render(&mut e, 2.0);
    frequency(&l[l.len() / 2..], expected)
}

#[test]
fn a2_is_110_hz_on_every_model() {
    for model in MODELS {
        let f = pitch(model, 9, 0, 0.0, 110.0);
        assert!(cents(f, 110.0).abs() < 1.0, "{model:?}: {f} Hz");
    }
}

#[test]
fn tune_moves_by_semitones() {
    for model in MODELS {
        for (tune, hz) in [
            (12, 220.0),
            (-12, 55.0),
            (7, 164.8138),
            (24, 440.0),
            (-24, 27.5),
        ] {
            let f = pitch(model, 9, tune, 0.0, hz);
            assert!(cents(f, hz).abs() < 3.0, "{model:?} tune {tune}: {f} Hz");
        }
    }
}

#[test]
fn fine_moves_by_cents() {
    for model in MODELS {
        let base = pitch(model, 9, 0, 0.0, 110.0);
        for fine in [-100.0, -37.0, 50.0, 100.0] {
            let f = pitch(model, 9, 0, fine, 110.0 * 2f64.powf(fine as f64 / 1200.0));
            let moved = cents(f, base);
            assert!(
                (moved - fine as f64).abs() < 1.5,
                "{model:?} fine {fine}: {moved} cents"
            );
        }
    }
}

/// The highest and lowest notes at the tune's ends are past the DW-303 DAC's range and at
/// the ends of ReBirth's pitch table.
#[test]
fn the_ends_of_the_range_stay_in_tune() {
    for model in MODELS {
        // C4 + 24 = C6, and C1 - 24 = C-1
        let f = pitch(model, NOTE_MAX, 24, 0.0, 1046.502);
        assert!(cents(f, 1046.502).abs() < 6.0, "{model:?} top: {f} Hz");
        let mut e = held(model, NOTE_MIN);
        e.params.voices[0].tune = -24;
        e.params.voices[0].fine = -100.0;
        e.play();
        let (l, r) = render(&mut e, 0.5);
        assert!(
            l.iter().chain(&r).all(|x| x.is_finite()),
            "{model:?} bottom"
        );
    }
}

#[test]
fn levels_match() {
    let level = |model| {
        let mut e = held(model, 0);
        e.patterns[0][0] = riff();
        let v = &mut e.params.voices[0];
        (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (0.5, 0.5, 0.5, 0.5, 0.5);
        e.play();
        let (l, _) = render(&mut e, 4.0);
        db(rms(&l))
    };
    let rb = level(Model::Rebirth);
    for model in [Model::RebirthVintage, Model::Dw303, Model::Open303] {
        let d = level(model) - rb;
        println!("{model:?}: {d:+.1} dB");
        assert!(
            d.abs() < 3.0,
            "{model:?} is {d:.1} dB from the ReBirth model"
        );
    }
}

#[test]
fn every_model_plays_and_stays_finite() {
    for model in MODELS {
        for wave in [Wave::Saw, Wave::Square] {
            let mut e = held(model, 0);
            e.patterns[0][0] = riff();
            let v = &mut e.params.voices[0];
            v.wave = wave;
            (v.cutoff, v.resonance, v.env_mod, v.accent) = (1.0, 1.0, 1.0, 1.0);
            e.play();
            let (l, r) = render(&mut e, 2.0);
            assert!(
                l.iter().chain(&r).all(|x| x.is_finite()),
                "{model:?} {wave:?}"
            );
            assert!(rms(&l) > 1e-3, "{model:?} {wave:?} is silent");
            assert!(peak(&l) < 4.0, "{model:?} {wave:?} peaks at {}", peak(&l));
        }
    }
}

#[test]
fn a_model_change_while_playing_is_clean() {
    let mut e = held(Model::Rebirth, 0);
    e.patterns[0][0] = riff();
    e.play();
    let mut all = Vec::new();
    for model in [
        Model::Dw303,
        Model::RebirthVintage,
        Model::Dw303,
        Model::Rebirth,
    ] {
        let (l, _) = render(&mut e, 0.7);
        all.extend(l);
        e.params.voices[0].model = model;
    }
    assert!(all.iter().all(|x| x.is_finite()));
    assert!(peak(&all) < 2.0);
}
