//! Step rates, MIDI transposition, the outputs, and the effect modes.
mod common;
use common::*;
use front303_core::*;

/// Steps voice 0 has started over `seconds`, counted from the pattern step reports.
fn steps_played(rate: Rate, seconds: f64) -> usize {
    let mut e = Engine::new();
    e.params.voices[0].rate = rate;
    e.patterns[0][0] = riff();
    e.params.bpm = 120.0;
    e.play();
    let mut count = 0;
    let mut last = None;
    let (mut l, mut r) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    for _ in 0..(seconds * SAMPLE_RATE / 64.0) as usize {
        e.render(&mut l, &mut r);
        let s = e.pattern_step(0);
        if s != last {
            count += 1;
            last = s;
        }
    }
    count
}

#[test]
fn every_voice_has_its_own_step_rate() {
    // 120 BPM: eight sixteenths a second
    for (rate, per_second) in [
        (Rate::Sixteenth, 8.0),
        (Rate::Eighth, 4.0),
        (Rate::ThirtySecond, 16.0),
        (Rate::SixteenthTriplet, 12.0),
        (Rate::Quarter, 2.0),
        (Rate::SixteenthDotted, 8.0 / 1.5),
    ] {
        let n = steps_played(rate, 4.0) as f64;
        assert!(
            (n - per_second * 4.0).abs() <= 1.0,
            "{rate:?}: {n} steps in 4 s"
        );
    }
}

#[test]
fn rates_stay_locked_to_the_bar() {
    // after two bars a 1/8T voice (six steps a bar) has played exactly twelve steps
    let mut e = Engine::new();
    e.params.voices[0].rate = Rate::EighthTriplet;
    e.patterns[0][0] = Pattern::from_steps(&[Step::note(0); 12]);
    e.params.bpm = 120.0;
    e.play();
    // two bars at 120 BPM are 4 s; render to just before the third bar's line
    let (mut l, mut r) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    for _ in 0..(4.0 * SAMPLE_RATE / 64.0 - 2.0) as usize {
        e.render(&mut l, &mut r);
    }
    assert_eq!(e.pattern_step(0), Some(11));
}

#[test]
fn midi_transposes_only_the_voices_that_follow_it() {
    let pitch_of = |follow: bool, transpose: i32| {
        let mut e = held(Model::Rebirth, 9);
        e.params.voices[0].midi_transpose = follow;
        e.transpose = transpose;
        e.play();
        let (l, _) = render(&mut e, 1.5);
        frequency(
            &l[l.len() / 2..],
            110.0 * 2f64.powf(transpose as f64 / 12.0),
        )
    };
    let up = pitch_of(true, 7);
    assert!(cents(up, 110.0 * 2f64.powf(7.0 / 12.0)).abs() < 3.0, "{up}");
    let down = pitch_of(true, -12);
    assert!(cents(down, 55.0).abs() < 3.0, "{down}");
    let ignored = pitch_of(false, 7);
    assert!(cents(ignored, 110.0).abs() < 3.0, "{ignored}");
}

fn render_all(e: &mut Engine, seconds: f64) -> Vec<Vec<f32>> {
    let n = (seconds * SAMPLE_RATE) as usize;
    let mut out = vec![vec![0.0f32; n]; OUTPUTS];
    let mut done = 0;
    while done < n {
        let k = 333.min(n - done);
        let mut slices: Vec<&mut [f32]> = out.iter_mut().map(|c| &mut c[done..done + k]).collect();
        e.render_outputs(&mut slices);
        done += k;
    }
    out
}

fn band() -> Engine {
    let mut e = Engine::new();
    for v in 0..VOICES {
        e.params.channels[v].on = true;
        e.params.channels[v].pan = [20, 64, 100, 127][v];
        e.params.channels[v].send = 50;
        e.params.channels[v].reverb = 0.4;
        e.patterns[v][0] = riff();
        e.params.voices[v].tune = 12 * (v as i8 % 2);
    }
    e.play();
    e
}

#[test]
fn the_stereo_mix_is_the_sum_of_the_stems_and_the_returns() {
    let mut e = band();
    e.params.master = 100;
    let out = render_all(&mut e, 2.0);
    // master gain at 100 and no master compressor: main = (stems + returns) · master gain
    let stems: Vec<f64> = (0..out[0].len())
        .map(|i| {
            (0..VOICES)
                .map(|v| out[voice_output(v)][i] as f64)
                .sum::<f64>()
                + out[FX_OUTPUT][i] as f64
        })
        .collect();
    // the master gain is one factor (it settles within the first blocks): fit it, then the
    // rest must vanish
    let skip = 4410;
    let main = &out[0][skip..];
    let s = &stems[skip..];
    let k = main.iter().zip(s).map(|(&m, &x)| m as f64 * x).sum::<f64>()
        / s.iter().map(|x| x * x).sum::<f64>();
    let residual = (main
        .iter()
        .zip(s)
        .map(|(&m, &x)| (m as f64 - k * x).powi(2))
        .sum::<f64>()
        / s.len() as f64)
        .sqrt();
    assert!(residual < 0.01 * rms(main), "residual {residual}, gain {k}");
    for v in 0..VOICES {
        let level = rms(&out[voice_output(v)]) + rms(&out[voice_output(v) + 1]);
        assert!(level > 1e-3, "voice {v} stem is silent");
    }
    assert!(rms(&out[FX_OUTPUT]) > 1e-4, "the returns are silent");
}

#[test]
fn multi_out_silences_the_main_pair() {
    let mut e = band();
    e.params.output = Output::Multi;
    let out = render_all(&mut e, 1.0);
    assert!(peak(&out[0]) == 0.0 && peak(&out[1]) == 0.0);
    for v in 0..VOICES {
        let level = rms(&out[voice_output(v)]) + rms(&out[voice_output(v) + 1]);
        assert!(level > 1e-3, "voice {v}: {level}");
    }
    // voice 0 is panned left, voice 3 hard right
    assert!(rms(&out[voice_output(0)]) > rms(&out[voice_output(0) + 1]));
    assert!(rms(&out[voice_output(3) + 1]) > 10.0 * rms(&out[voice_output(3)]));
}

/// One voice through a setting of the effects; the main output.
fn through(set: impl Fn(&mut Engine)) -> (Vec<f32>, Vec<f32>) {
    let mut e = Engine::new();
    e.patterns[0][0] = riff();
    e.params.channels[0].send = 90;
    e.params.channels[0].reverb = 0.6;
    set(&mut e);
    e.play();
    render(&mut e, 3.0)
}

#[test]
fn every_effect_mode_runs_clean() {
    type Setting = Box<dyn Fn(&mut Engine)>;
    let modes: Vec<(&str, Setting)> = vec![
        (
            "setekh",
            Box::new(|e: &mut Engine| {
                e.params.channels[0].dist = true;
                e.params.dist.mode = DistMode::Setekh;
                e.params.dist.amount = 127;
                e.params.dist.shape = 64;
            }),
        ),
        (
            "valentine on the channel",
            Box::new(|e: &mut Engine| {
                e.params.comp.mode = CompMode::Valentine;
                e.params.comp.route = CompRoute::Channel(0);
                e.params.comp.valentine.saturate = 6;
                e.params.comp.valentine.crush = 4;
            }),
        ),
        (
            "valentine on the master",
            Box::new(|e: &mut Engine| {
                e.params.comp.mode = CompMode::Valentine;
                e.params.comp.route = CompRoute::Master;
            }),
        ),
        (
            "tape",
            Box::new(|e: &mut Engine| {
                e.params.delay.mode = DelayMode::Tape;
                e.params.delay.feedback = 100;
                e.params.delay.a = 80;
                e.params.delay.b = 80;
            }),
        ),
        (
            "intervals",
            Box::new(|e: &mut Engine| {
                e.params.delay.mode = DelayMode::Intervals;
                e.params.delay.kind = 3;
                e.params.delay.feedback = 60;
                e.params.delay.b = 20;
            }),
        ),
    ];
    for (name, set) in modes {
        let (l, r) = through(|e| set(e));
        assert!(l.iter().chain(&r).all(|x| x.is_finite()), "{name}");
        assert!(rms(&l) > 1e-3, "{name} is silent");
        assert!(peak(&l).max(peak(&r)) < 8.0, "{name} peaks at {}", peak(&l));
    }
    for mode in ReverbMode::ALL {
        for program in 0..8 {
            let (l, r) = through(|e| {
                e.params.reverb.mode = mode;
                e.params.reverb.program = program;
                e.params.reverb.size = 1.0;
                e.params.reverb.decay = 1.0;
            });
            assert!(
                l.iter().chain(&r).all(|x| x.is_finite()),
                "{mode:?} {program}"
            );
            assert!(peak(&l).max(peak(&r)) < 8.0, "{mode:?} {program}");
        }
    }
}

#[test]
fn the_tape_echo_repeats_at_its_time() {
    // one short note; the echo returns one step (125 ms at 120 BPM) later
    let mut e = Engine::new();
    e.params.bpm = 120.0;
    e.patterns[0][0] = Pattern::from_steps(&[
        Step::note(12).accent(),
        Step::default(),
        Step::default(),
        Step::default(),
        Step::default(),
        Step::default(),
        Step::default(),
        Step::default(),
    ]);
    e.params.channels[0].send = 127;
    e.params.delay.mode = DelayMode::Tape;
    e.params.delay.steps = 2;
    e.params.delay.feedback = 0;
    e.params.delay.a = 0;
    e.params.delay.b = 0;
    e.params.delay.c = 0;
    e.params.reverb.on = false;
    // the transport's inertia takes the tape to speed: let it settle first
    let (mut l, mut r) = (vec![0.0f32; 44100], vec![0.0f32; 44100]);
    for _ in 0..6 {
        e.render(&mut l, &mut r);
    }
    e.play();
    let out = render_all(&mut e, 0.8);
    let fx = &out[FX_OUTPUT];
    let onset = fx.iter().position(|x| x.abs() > 0.02).expect("an echo");
    let t = onset as f64 / SAMPLE_RATE;
    assert!((t - 0.25).abs() < 0.02, "echo at {t} s");
}

#[test]
fn valentine_compresses() {
    let level = |route| {
        let mut e = held(Model::Rebirth, 0);
        e.params.voices[0].cutoff = 0.8;
        e.params.comp.mode = CompMode::Valentine;
        e.params.comp.route = route;
        e.params.comp.amount = 127; // +36 dB into the compressor
        e.params.comp.threshold = 85; // 0 dB out
        e.params.comp.valentine.ratio = 21;
        e.params.reverb.on = false;
        e.play();
        let (l, _) = render(&mut e, 1.0);
        db(rms(&l[22050..]))
    };
    let off = level(CompRoute::Off);
    let on = level(CompRoute::Channel(0));
    // 36 dB more drive into a near-limiter: far less than 36 dB louder
    assert!(on - off < 12.0, "off {off:.1} dB, on {on:.1} dB");
}
