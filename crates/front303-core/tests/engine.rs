//! The mixer, the effects' routing and the transport.
mod common;
use common::*;
use front303_core::*;

fn playing(model: Model) -> Engine {
    let mut e = held(model, 0);
    e.patterns[0][0] = riff();
    let v = &mut e.params.voices[0];
    (v.cutoff, v.resonance, v.env_mod) = (0.5, 0.6, 0.5);
    e.play();
    e
}

fn differs(a: &[f32], b: &[f32]) -> bool {
    a.iter().zip(b).any(|(x, y)| (x - y).abs() > 1e-6)
}

#[test]
fn the_output_does_not_depend_on_the_callers_buffer_size() {
    for model in [Model::Rebirth, Model::Dw303] {
        let mut reference = playing(model);
        let n = 20_000;
        let (mut l0, mut r0) = (vec![0.0; n], vec![0.0; n]);
        reference.render(&mut l0, &mut r0);
        for size in [1, 63, 64, 65, 512, 4096] {
            let mut e = playing(model);
            let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
            for (a, b) in l.chunks_mut(size).zip(r.chunks_mut(size)) {
                e.render(a, b);
            }
            assert!(l == l0 && r == r0, "{model:?} in buffers of {size}");
        }
    }
}

#[test]
fn stop_closes_the_gates() {
    for model in MODELS {
        let mut e = playing(model);
        render(&mut e, 1.0);
        e.stop();
        let (l, _) = render(&mut e, 4.0);
        let tail = &l[l.len() * 3 / 4..];
        assert!(
            db(rms(tail)) < -80.0,
            "{model:?} rings at {:.1} dB",
            db(rms(tail))
        );
    }
}

#[test]
fn a_channel_that_is_off_is_silent() {
    let mut e = playing(Model::Rebirth);
    e.params.channels[0].on = false;
    let (l, _) = render(&mut e, 1.0);
    assert!(db(rms(&l[l.len() / 2..])) < -90.0);
}

#[test]
fn pan_moves_the_voice() {
    let mut e = playing(Model::Rebirth);
    e.params.channels[0].pan = 0;
    let (l, r) = render(&mut e, 1.0);
    assert!(rms(&l) > 100.0 * rms(&r));
    let mut e = playing(Model::Rebirth);
    e.params.channels[0].pan = 127;
    let (l, r) = render(&mut e, 1.0);
    assert!(rms(&r) > 10.0 * rms(&l));
}

#[test]
fn the_distortion_follows_the_channel_switch() {
    let (clean, _) = render(&mut playing(Model::Rebirth), 1.0);
    for shape in [0, 64, 127] {
        let mut e = playing(Model::Rebirth);
        e.params.dist.shape = shape;
        // the unit alone changes nothing: the channel's switch routes it
        let (same, _) = render(&mut e, 1.0);
        assert!(!differs(&clean, &same), "shape {shape} without the switch");
        let mut e = playing(Model::Rebirth);
        e.params.dist.shape = shape;
        e.params.channels[0].dist = true;
        let (dirty, _) = render(&mut e, 1.0);
        assert!(differs(&clean, &dirty), "shape {shape}");
        assert!(dirty.iter().all(|x| x.is_finite()));
    }
}

#[test]
fn the_pcf_sits_on_one_channel() {
    let (clean, _) = render(&mut playing(Model::Rebirth), 1.0);
    for mode in [PcfMode::Lowpass, PcfMode::Bandpass] {
        let mut e = playing(Model::Rebirth);
        e.params.pcf.mode = mode;
        e.params.pcf.channel = Some(1);
        let (other, _) = render(&mut e, 1.0);
        assert!(!differs(&clean, &other), "{mode:?} on another channel");
        e = playing(Model::Rebirth);
        e.params.pcf.mode = mode;
        e.params.pcf.channel = Some(0);
        let (filtered, _) = render(&mut e, 1.0);
        assert!(differs(&clean, &filtered), "{mode:?}");
        assert!(rms(&filtered) > 1e-4, "{mode:?} is silent");
    }
}

#[test]
fn every_pcf_pattern_runs() {
    for pattern in 0..=255u8 {
        let mut e = playing(Model::Rebirth);
        e.params.pcf.channel = Some(0);
        e.params.pcf.pattern = pattern;
        let (l, _) = render(&mut e, 0.3);
        assert!(l.iter().all(|x| x.is_finite()), "pattern {pattern}");
        assert!(rms(&l) > 1e-5, "pattern {pattern} is silent");
    }
}

#[test]
fn the_compressor_works_on_a_channel_and_on_the_master() {
    let (clean, _) = render(&mut playing(Model::Rebirth), 1.5);
    for route in [CompRoute::Channel(0), CompRoute::Master] {
        let mut e = playing(Model::Rebirth);
        e.params.comp.route = route;
        e.params.comp.amount = 127;
        let (squeezed, _) = render(&mut e, 1.5);
        assert!(differs(&clean, &squeezed), "{route:?}");
        assert!(rms(&squeezed) > 1e-4 && squeezed.iter().all(|x| x.is_finite()));
    }
    let mut e = playing(Model::Rebirth);
    e.params.comp.route = CompRoute::Channel(2);
    let (other, _) = render(&mut e, 1.5);
    assert!(!differs(&clean, &other));
}

#[test]
fn the_delay_repeats_what_the_send_feeds_it() {
    let tail = |send: u8, on: bool| {
        let mut e = playing(Model::Rebirth);
        e.params.channels[0].send = send;
        e.params.delay.on = on;
        e.params.delay.feedback = 100;
        render(&mut e, 2.0);
        e.stop();
        let (l, _) = render(&mut e, 1.5);
        db(rms(&l[l.len() / 2..]))
    };
    assert!(
        tail(100, true) > -60.0,
        "no echo: {:.1} dB",
        tail(100, true)
    );
    assert!(tail(0, true) < -80.0, "an echo without a send");
    assert!(tail(100, false) < -80.0, "an echo with the delay off");
}

#[test]
fn four_voices_mix() {
    let mut e = Engine::new();
    for c in 0..4 {
        e.patterns[c][0] = riff();
        e.params.voices[c].tune = [0, 12, -12, 7][c];
        e.params.voices[c].model = MODELS[c % 4];
        e.params.channels[c].pan = [20, 100, 64, 64][c];
    }
    e.play();
    let (l, r) = render(&mut e, 2.0);
    assert!(l.iter().chain(&r).all(|x| x.is_finite()));
    assert!(rms(&l) > 1e-3 && rms(&r) > 1e-3);
    // a pattern of another length loops on its own
    e.patterns[1][0].len = 5;
    render(&mut e, 1.0);
}
