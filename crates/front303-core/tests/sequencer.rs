//! The generator, the scale quantisation, the pattern slots, shuffle and host sync.
mod common;
use common::*;
use front303_core::generator::{generate, mutate, quantize, rotate, transpose};
use front303_core::*;

fn in_scale(note: i8, root: u8, scale: Scale) -> bool {
    scale
        .intervals()
        .contains(&((note as i32 - root as i32).rem_euclid(12) as u8))
}

#[test]
fn quantize_lands_in_the_scale_and_stays_close() {
    for scale in Scale::ALL {
        for root in 0..12 {
            for note in NOTE_MIN..=NOTE_MAX {
                let q = quantize(note, root, scale);
                assert!(
                    in_scale(q, root, scale),
                    "{scale:?} root {root} note {note} -> {q}"
                );
                assert!((NOTE_MIN..=NOTE_MAX).contains(&q));
                assert!(
                    (q - note).abs() <= 3,
                    "{scale:?} root {root} note {note} -> {q}"
                );
                if in_scale(note, root, scale) {
                    assert_eq!(q, note);
                }
            }
        }
    }
}

#[test]
fn generated_patterns_are_in_scale_and_follow_the_settings() {
    let mut rng = Rng::new(1);
    for scale in Scale::ALL {
        for root in [0, 3, 7, 11] {
            let g = GenParams::default();
            let mut played = 0;
            for _ in 0..50 {
                let mut p = Pattern::default();
                generate(&mut p, &g, root, scale, &mut rng);
                for (i, s) in p.steps.iter().enumerate() {
                    assert!(
                        in_scale(s.note, root, scale),
                        "{scale:?} {root}: {}",
                        s.note
                    );
                    assert!((NOTE_MIN..=NOTE_MAX).contains(&s.note));
                    let next = p.steps[(i + 1) % 16];
                    assert!(!(s.slide && (s.rest || next.rest)), "a slide into nothing");
                    played += !s.rest as usize;
                }
            }
            // density 0.75 of 800 steps
            assert!(
                (480..720).contains(&played),
                "{scale:?}: {played} of 800 steps play"
            );
        }
    }
}

#[test]
fn the_settings_shape_the_line() {
    let count = |g: GenParams, f: fn(&Step) -> bool| {
        let mut rng = Rng::new(7);
        let mut n = 0;
        for _ in 0..100 {
            let mut p = Pattern::default();
            generate(&mut p, &g, 0, Scale::Minor, &mut rng);
            n += p.steps.iter().filter(|s| f(s)).count();
        }
        n
    };
    let d = GenParams::default();
    let played = |s: &Step| !s.rest;
    assert!(count(GenParams { density: 1.0, ..d }, played) == 1600);
    assert!(count(GenParams { density: 0.2, ..d }, played) < 600);
    assert!(count(GenParams { accents: 0.0, ..d }, |s| s.accent) == 0);
    assert!(count(GenParams { accents: 0.8, ..d }, |s| s.accent) > 500);
    assert!(count(GenParams { slides: 0.0, ..d }, |s| s.slide) == 0);
    assert!(
        count(GenParams { octaves: 0.0, ..d }, |s| !(0..12)
            .contains(&s.note))
            == 0
    );
    assert!(
        count(GenParams { octaves: 1.0, ..d }, |s| !(0..12)
            .contains(&s.note))
            > 400
    );
}

#[test]
fn a_seed_gives_the_same_pattern_and_another_seed_another() {
    let make = |seed| {
        let mut p = Pattern::default();
        generate(
            &mut p,
            &GenParams::default(),
            2,
            Scale::Dorian,
            &mut Rng::new(seed),
        );
        p
    };
    assert_eq!(make(42), make(42));
    assert_ne!(make(42), make(43));
}

#[test]
fn mutate_changes_about_the_share_it_is_asked_to() {
    let mut rng = Rng::new(3);
    let g = GenParams::default();
    let mut original = Pattern::default();
    generate(&mut original, &g, 0, Scale::Minor, &mut rng);
    let mut same = original;
    mutate(&mut same, &g, 0.0, 0, Scale::Minor, &mut rng);
    assert_eq!(same, original);
    let mut changed = 0;
    for _ in 0..200 {
        let mut p = original;
        mutate(&mut p, &g, 0.25, 0, Scale::Minor, &mut rng);
        changed += p
            .steps
            .iter()
            .zip(&original.steps)
            .filter(|(a, b)| a != b)
            .count();
    }
    // a quarter of 3200 steps are touched; some rerolls give back what was there
    assert!((250..900).contains(&changed), "{changed}");
}

#[test]
fn rotate_and_transpose() {
    let mut p = riff();
    let first = p.steps[0];
    rotate(&mut p, 3);
    assert_eq!(p.steps[3], first);
    rotate(&mut p, -3);
    assert_eq!(p, riff());
    transpose(&mut p, 7);
    assert_eq!(p.steps[0].note, 7);
    transpose(&mut p, 24);
    assert!(
        p.steps
            .iter()
            .all(|s| (NOTE_MIN..=NOTE_MAX).contains(&s.note))
    );
}

#[test]
fn the_scale_quantises_what_plays() {
    // an F sharp held in C major sounds an F (a tie goes down)
    let mut e = held(Model::Rebirth, 6);
    e.params.scale = Scale::Major;
    e.params.voices[0].cutoff = 0.8;
    e.params.voices[0].resonance = 0.0;
    e.play();
    let (l, _) = render(&mut e, 2.0);
    let f = frequency(&l[l.len() / 2..], 87.307);
    assert!(cents(f, 87.307).abs() < 3.0, "{f} Hz");
}

#[test]
fn a_pattern_change_waits_for_the_bar_line() {
    let mut e = held(Model::Rebirth, 0);
    e.patterns[0][1] = Pattern::from_steps(&[Step::note(12).slide()]);
    e.play();
    render(&mut e, 0.2);
    e.params.voices[0].pattern = 1;
    assert_eq!(e.active_pattern(0), 0);
    // a bar of 16 steps at 125 BPM is 1.92 s
    render(&mut e, 1.0);
    assert_eq!(e.active_pattern(0), 0);
    render(&mut e, 1.0);
    assert_eq!(e.active_pattern(0), 1);
    // stopped, it changes at once
    e.stop();
    e.params.voices[0].pattern = 0;
    render(&mut e, 0.01);
    assert_eq!(e.active_pattern(0), 0);
}

#[test]
fn evolve_rewrites_the_playing_pattern_each_time_round() {
    let mut e = held(Model::Rebirth, 0);
    e.patterns[0][0] = riff();
    e.params.scale = Scale::Minor;
    e.play();
    render(&mut e, 4.5);
    assert_eq!(e.patterns[0][0], riff());
    assert_eq!(e.evolutions[0], 0);
    e.params.voices[0].generator.evolve = 0.5;
    render(&mut e, 4.0);
    assert!(e.evolutions[0] >= 2);
    assert_ne!(e.patterns[0][0], riff());
    assert_eq!(e.patterns[1][0], Pattern::default());
}

/// The time of each note start in `x`: where the level comes back after 30 ms of quiet.
fn onsets(x: &[f32]) -> Vec<f64> {
    let hop = 64;
    let mut out = Vec::new();
    let mut quiet = 0;
    for (i, c) in x.chunks(hop).enumerate() {
        if db(rms(c)) < -60.0 {
            quiet += 1;
        } else {
            if quiet > 20 {
                out.push((i * hop) as f64 / SAMPLE_RATE);
            }
            quiet = 0;
        }
    }
    out
}

#[test]
fn shuffle_delays_every_second_step() {
    // at 60 BPM a step is 250 ms; the pattern plays one step in two
    let first = |odd: bool, shuffle: f32| {
        let mut e = held(Model::Rebirth, 0);
        let (note, rest) = (Step::note(0), Step::default());
        let steps = if odd { [rest, note] } else { [note, rest] };
        e.patterns[0][0] = Pattern::from_steps(&steps);
        e.params.voices[0].cutoff = 0.6;
        e.params.bpm = 60.0;
        e.params.shuffle = shuffle;
        e.play();
        let (l, _) = render(&mut e, 2.2);
        let t = onsets(&l);
        assert!(t.len() >= 3, "{} onsets", t.len());
        (t[0], t[1] - t[0])
    };
    for shuffle in [0.0, 0.5, 1.0] {
        // the even steps stay where they are
        let (at, period) = first(false, shuffle);
        assert!(
            (at - 0.5).abs() < 0.006 && (period - 0.5).abs() < 0.006,
            "even {at} {period}"
        );
        // the odd steps come late by up to half a step
        let (at, period) = first(true, shuffle);
        let late = 0.25 + 0.125 * shuffle as f64;
        assert!(
            (at - late).abs() < 0.006,
            "shuffle {shuffle}: {at}, expected {late}"
        );
        assert!((period - 0.5).abs() < 0.006);
    }
}

#[test]
fn sync_follows_a_host() {
    let mut e = held(Model::Rebirth, 0);
    e.patterns[0][0] = riff();
    e.params.bpm = 120.0;
    // a host playing from bar 3, in buffers of 512: the engine stays on the host's step
    let step_samples = SAMPLE_RATE * 60.0 / 120.0 / 4.0;
    let mut buf = [0.0f32; 512];
    let mut right = [0.0f32; 512];
    for n in 0..200 {
        let host = 32.0 + (n * 512) as f64 / step_samples;
        e.sync(true, host);
        e.render(&mut buf, &mut right);
        let end = host + 512.0 / step_samples;
        let step = e.current_step().expect("playing");
        assert!(
            (step as f64 - end.floor()).abs() <= 1.0,
            "block {n}: step {step}, host {end}"
        );
    }
    // the host loops back: the engine goes with it
    e.sync(true, 0.0);
    e.render(&mut buf, &mut right);
    assert_eq!(e.current_step(), Some(0));
    assert_eq!(e.pattern_step(0), Some(0));
    e.sync(false, 0.1);
    assert!(!e.is_playing());
}
