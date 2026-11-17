//! `front303 demo DIR`: renders Front 303's demonstration files as 16-bit WAVs.
//!
//! - `compare_<model>.wav`: one line on one voice, clean, with a slow cutoff sweep, once per
//!   model, so the three models can be heard side by side;
//! - `demo_4x303.wav`: four voices on different models through the distortion, the PCF, the
//!   compressor and the delay;
//! - `demo_generative.wav`: generated lines that evolve as they play;
//! - `demo_effects.wav`: Open303 and the other models at four step rates under one tempo,
//!   transposed bar by bar as MIDI notes would, through Setekh, Valentine, the tape and
//!   interval delays and the four reverb engines in turn;
//! - `stems/`: the effects demo again in the multi-output mode, one file per output pair.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use front303_core::{
    CompMode, CompRoute, DelayMode, DistMode, Engine, FX_OUTPUT, GenParams, Model, OUTPUTS, Output,
    Pattern, PcfMode, Rate, ReverbMode, Rng, SAMPLE_RATE, Scale, Step, VOICES, Wave, generator,
    voice_output,
};

fn write_wav(path: &Path, l: &[f32], r: &[f32]) -> io::Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    let data = (l.len() * 4) as u32;
    let rate = SAMPLE_RATE as u32;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&rate.to_le_bytes())?;
    f.write_all(&(rate * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data.to_le_bytes())?;
    for (&a, &b) in l.iter().zip(r) {
        for x in [a, b] {
            let v = (x.clamp(-1.0, 1.0) * 32767.0).round() as i16;
            f.write_all(&v.to_le_bytes())?;
        }
    }
    f.flush()
}

/// Renders `seconds`, calling `automate` with the time before every 64 samples.
fn render(
    e: &mut Engine,
    seconds: f64,
    mut automate: impl FnMut(&mut Engine, f64),
) -> (Vec<f32>, Vec<f32>) {
    let n = (seconds * SAMPLE_RATE) as usize;
    let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
    let mut done = 0;
    for (a, b) in l.chunks_mut(64).zip(r.chunks_mut(64)) {
        automate(e, done as f64 / SAMPLE_RATE);
        e.render(a, b);
        done += a.len();
    }
    (l, r)
}

fn report(name: &str, l: &[f32], r: &[f32], seconds: f64, took: f64) {
    let peak = l.iter().chain(r).fold(0.0f32, |m, x| m.max(x.abs()));
    let rms = (l.iter().chain(r).map(|&x| x as f64 * x as f64).sum::<f64>() / (2 * l.len()) as f64)
        .sqrt();
    println!(
        "{name}: {seconds:.0} s, peak {:.1} dBFS, rms {:.1} dBFS, rendered in {:.0}% of real time",
        20.0 * (peak as f64).max(1e-9).log10(),
        20.0 * rms.max(1e-9).log10(),
        took / seconds * 100.0
    );
}

fn acid_line() -> Pattern {
    let n = Step::note;
    Pattern::from_steps(&[
        n(0),
        n(0).accent(),
        n(12),
        n(0),
        n(3).slide(),
        n(5),
        n(0).accent(),
        n(-12),
        n(0),
        n(7).slide(),
        n(10).accent(),
        n(0),
        n(12),
        n(0).accent(),
        n(3).slide(),
        n(5),
    ])
}

fn bass_line() -> Pattern {
    let n = Step::note;
    let rest = Step::default();
    Pattern::from_steps(&[
        n(-12).accent(),
        rest,
        n(-12),
        n(0),
        rest,
        n(-12),
        n(-9).slide(),
        n(-12),
        n(-12).accent(),
        rest,
        n(-12),
        n(0),
        rest,
        n(-5).slide(),
        n(-7).accent(),
        n(-12),
    ])
}

fn high_line() -> Pattern {
    let n = Step::note;
    let rest = Step::default();
    Pattern::from_steps(&[
        n(12),
        rest,
        n(15).accent(),
        n(12),
        n(19),
        rest,
        n(12).slide(),
        n(24).accent(),
        rest,
        n(15),
        n(12),
    ])
}

fn pulse_line() -> Pattern {
    let n = Step::note;
    let rest = Step::default();
    Pattern::from_steps(&[n(0), rest, rest, n(0).accent(), rest, rest, n(7), rest])
}

/// A triangle 0..1..0 over `period` seconds.
fn sweep(t: f64, period: f64) -> f32 {
    let x = (t / period).fract();
    (1.0 - (2.0 * x - 1.0).abs()) as f32
}

fn compare(dir: &Path) -> io::Result<()> {
    for (model, name) in [
        (Model::Rebirth, "rebirth"),
        (Model::RebirthVintage, "rebirth_vintage"),
        (Model::Dw303, "dw303"),
        (Model::Open303, "open303"),
    ] {
        let mut e = Engine::new();
        for c in 1..4 {
            e.params.channels[c].on = false;
        }
        e.params.bpm = 128.0;
        e.params.channels[0].level = 104;
        e.patterns[0][0] = acid_line();
        let v = &mut e.params.voices[0];
        v.model = model;
        (v.resonance, v.env_mod, v.decay, v.accent) = (0.75, 0.55, 0.45, 0.8);
        e.play();
        let seconds = 15.0;
        let start = Instant::now();
        let (l, r) = render(&mut e, seconds, |e, t| {
            e.params.voices[0].cutoff = 0.12 + 0.6 * sweep(t, 15.0);
        });
        report(
            &format!("compare_{name}.wav"),
            &l,
            &r,
            seconds,
            start.elapsed().as_secs_f64(),
        );
        write_wav(&dir.join(format!("compare_{name}.wav")), &l, &r)?;
    }
    Ok(())
}

fn four(dir: &Path) -> io::Result<()> {
    let mut e = Engine::new();
    e.params.bpm = 128.0;
    e.params.master = 96;
    for (v, p) in [bass_line(), acid_line(), high_line(), pulse_line()]
        .into_iter()
        .enumerate()
    {
        e.patterns[v][0] = p;
    }

    // 1: the bass, the circuit model, dry and centred
    let v = &mut e.params.voices[0];
    v.model = Model::Dw303;
    (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (0.3, 0.55, 0.45, 0.35, 0.7);
    e.params.channels[0].level = 112;

    // 2: the lead, ReBirth through the distortion and into the delay
    let v = &mut e.params.voices[1];
    v.model = Model::Rebirth;
    v.tune = 12;
    (v.resonance, v.env_mod, v.decay, v.accent) = (0.85, 0.6, 0.5, 0.9);
    let c = &mut e.params.channels[1];
    (c.level, c.pan, c.send, c.dist) = (84, 44, 88, true);
    e.params.dist.amount = 60;
    e.params.dist.shape = 48;

    // 3: an eleven-step line an octave up, ReBirth Vintage, square, through the PCF
    let v = &mut e.params.voices[2];
    v.model = Model::RebirthVintage;
    v.wave = Wave::Square;
    v.tune = 12;
    v.fine = 7.0;
    (v.cutoff, v.resonance, v.env_mod, v.decay) = (0.55, 0.4, 0.3, 0.3);
    let c = &mut e.params.channels[2];
    (c.level, c.pan, c.send) = (86, 92, 70);
    e.params.pcf.channel = Some(2);
    e.params.pcf.mode = PcfMode::Bandpass;
    e.params.pcf.pattern = 12;
    e.params.pcf.amount = 90;

    // 4: a fifth-tuned pulse, the circuit model again, square
    let v = &mut e.params.voices[3];
    v.model = Model::Dw303;
    v.wave = Wave::Square;
    v.tune = -5;
    (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (0.4, 0.8, 0.7, 0.6, 1.0);
    let c = &mut e.params.channels[3];
    (c.level, c.pan, c.send) = (96, 78, 60);

    e.params.delay.steps = 3;
    e.params.delay.feedback = 76;
    e.params.comp.route = CompRoute::Master;
    e.params.comp.amount = 50;

    e.play();
    let seconds = 30.0;
    let start = Instant::now();
    let (l, r) = render(&mut e, seconds, |e, t| {
        e.params.voices[1].cutoff = 0.1 + 0.55 * sweep(t, 30.0);
        e.params.voices[0].cutoff = 0.25 + 0.15 * sweep(t, 7.5);
        e.params.pcf.frequency = (40.0 + 60.0 * sweep(t, 15.0)) as u8;
        if t > 28.0 {
            e.stop();
        }
    });
    report(
        "demo_4x303.wav",
        &l,
        &r,
        seconds,
        start.elapsed().as_secs_f64(),
    );
    write_wav(&dir.join("demo_4x303.wav"), &l, &r)
}

/// Four generated lines in D Phrygian, with shuffle; the second voice evolves as it plays.
fn generative(dir: &Path) -> io::Result<()> {
    let mut e = Engine::new();
    e.params.bpm = 126.0;
    e.params.shuffle = 0.35;
    e.params.root = 2;
    e.params.scale = Scale::Phrygian;
    e.params.master = 96;
    let mut rng = Rng::new(0x5EED);
    let settings = [
        (0.85, 0.3, 0.2, 0.15, -12, Model::Dw303, 0.35, 0.5),
        (0.7, 0.4, 0.35, 0.5, 0, Model::Rebirth, 0.2, 0.75),
        (0.45, 0.2, 0.3, 0.6, 12, Model::RebirthVintage, 0.5, 0.35),
        (0.6, 0.5, 0.15, 0.3, 0, Model::Dw303, 0.25, 0.8),
    ];
    for (v, &(density, accents, slides, octaves, tune, model, cutoff, reso)) in
        settings.iter().enumerate()
    {
        let g = GenParams {
            density,
            accents,
            slides,
            octaves,
            evolve: 0.0,
        };
        let mut p = Pattern {
            len: [16, 16, 12, 7][v],
            ..Pattern::default()
        };
        generator::generate(&mut p, &g, 2, Scale::Phrygian, &mut rng);
        e.patterns[v][0] = p;
        let vp = &mut e.params.voices[v];
        vp.generator = g;
        vp.model = model;
        vp.tune = tune;
        (vp.cutoff, vp.resonance, vp.env_mod, vp.decay, vp.accent) = (cutoff, reso, 0.5, 0.45, 0.8);
        let c = &mut e.params.channels[v];
        c.pan = [64, 40, 92, 70][v];
        c.level = [110, 92, 80, 84][v];
        c.send = [0, 70, 80, 50][v];
    }
    e.params.voices[1].generator.evolve = 0.15;
    e.params.voices[3].wave = Wave::Square;
    e.params.channels[1].dist = true;
    e.params.dist.amount = 50;
    e.params.dist.shape = 40;
    e.params.delay.steps = 3;
    e.params.delay.feedback = 70;
    e.play();
    let seconds = 32.0;
    let start = Instant::now();
    let (l, r) = render(&mut e, seconds, |e, t| {
        e.params.voices[1].cutoff = 0.15 + 0.5 * sweep(t, 16.0);
        if t > 30.0 {
            e.stop();
        }
    });
    report(
        "demo_generative.wav",
        &l,
        &r,
        seconds,
        start.elapsed().as_secs_f64(),
    );
    write_wav(&dir.join("demo_generative.wav"), &l, &r)
}

/// The effects demo's engine, before it plays.
fn effects_engine() -> Engine {
    let mut e = Engine::new();
    e.params.bpm = 124.0;
    e.params.master = 92;
    e.params.root = 9;
    e.params.scale = Scale::Minor;
    let mut rng = Rng::new(0xF3_0303);
    // four lines at four step rates: 1/16, 1/8T, 1/8. and 1/32
    let setup = [
        (Model::Open303, Rate::Sixteenth, -12, 0.35, 0.7, 16),
        (Model::Rebirth, Rate::EighthTriplet, 0, 0.3, 0.8, 12),
        (Model::Dw303, Rate::EighthDotted, 12, 0.45, 0.5, 5),
        (Model::RebirthVintage, Rate::ThirtySecond, 12, 0.5, 0.6, 16),
    ];
    for (v, &(model, rate, tune, cutoff, reso, len)) in setup.iter().enumerate() {
        let g = GenParams {
            density: [0.85, 0.7, 0.9, 0.45][v],
            accents: 0.35,
            slides: [0.3, 0.2, 0.1, 0.25][v],
            octaves: [0.3, 0.4, 0.2, 0.5][v],
            evolve: 0.0,
        };
        let mut p = Pattern {
            len,
            ..Pattern::default()
        };
        generator::generate(&mut p, &g, 9, Scale::Minor, &mut rng);
        e.patterns[v][0] = p;
        let vp = &mut e.params.voices[v];
        vp.model = model;
        vp.rate = rate;
        vp.tune = tune;
        // the 1/32 line stays put while the others follow the transposition
        vp.midi_transpose = v != 3;
        (vp.cutoff, vp.resonance, vp.env_mod, vp.decay, vp.accent) = (cutoff, reso, 0.5, 0.4, 0.8);
        let c = &mut e.params.channels[v];
        c.on = true;
        c.level = [112, 86, 74, 70][v];
        c.pan = [64, 38, 96, 80][v];
        c.send = [0, 80, 70, 40][v];
        c.reverb = [0.1, 0.45, 0.6, 0.35][v];
    }
    e.params.voices[3].wave = Wave::Square;
    // Setekh on the triplet line, Valentine on the master
    e.params.channels[1].dist = true;
    e.params.dist.mode = DistMode::Setekh;
    e.params.dist.amount = 90;
    e.params.dist.shape = 40;
    e.params.dist.level = 54;
    e.params.comp.mode = CompMode::Valentine;
    e.params.comp.route = CompRoute::Master;
    e.params.comp.amount = 28;
    e.params.comp.threshold = 80;
    e.params.comp.valentine.saturate = 3;
    e.params.delay.steps = 3;
    e.params.delay.feedback = 70;
    e.params.reverb.level = 0.8;
    e
}

/// The effects demo's automation: a new delay mode and reverb engine every eight bars of
/// 7.74 s, and a transposition per bar.
fn effects_automation(e: &mut Engine, t: f64) {
    let bar = 4.0 * 60.0 / 124.0;
    let section = (t / (8.0 * bar)) as usize;
    let b = (t / bar) as usize;
    e.transpose = [0, 0, 5, 3, 0, 0, 7, -2][b % 8];
    e.params.delay.mode = [
        DelayMode::Tape,
        DelayMode::Intervals,
        DelayMode::Tape,
        DelayMode::Rebirth,
    ][section.min(3)];
    e.params.delay.kind = [5, 3, 10, 0][section.min(3)];
    e.params.delay.a = [40, 30, 90, 30][section.min(3)];
    e.params.delay.b = [30, 40, 60, 30][section.min(3)];
    e.params.reverb.mode = ReverbMode::ALL[section.min(3)];
    e.params.reverb.program = [4, 2, 5, 1][section.min(3)];
    e.params.voices[0].cutoff = 0.25 + 0.35 * sweep(t, 16.0);
    e.params.voices[1].cutoff = 0.2 + 0.5 * sweep(t, 12.0);
    if t > 8.0 * 4.0 * bar - 1.5 {
        e.stop();
    }
}

fn effects(dir: &Path) -> io::Result<()> {
    let mut e = effects_engine();
    e.play();
    let seconds = 8.0 * 4.0 * 4.0 * 60.0 / 124.0 + 2.5;
    let start = Instant::now();
    let (l, r) = render(&mut e, seconds, effects_automation);
    report(
        "demo_effects.wav",
        &l,
        &r,
        seconds,
        start.elapsed().as_secs_f64(),
    );
    write_wav(&dir.join("demo_effects.wav"), &l, &r)?;

    // the same in the multi-output mode
    let mut e = effects_engine();
    e.params.output = Output::Multi;
    e.play();
    let n = (seconds * SAMPLE_RATE) as usize;
    let mut out = vec![vec![0.0f32; n]; OUTPUTS];
    let mut done = 0;
    while done < n {
        let k = 64.min(n - done);
        effects_automation(&mut e, done as f64 / SAMPLE_RATE);
        let mut slices: Vec<&mut [f32]> = out.iter_mut().map(|c| &mut c[done..done + k]).collect();
        e.render_outputs(&mut slices);
        done += k;
    }
    let stems = dir.join("stems");
    std::fs::create_dir_all(&stems)?;
    for v in 0..VOICES {
        let o = voice_output(v);
        let name = format!("303-{}.wav", v + 1);
        report(&format!("stems/{name}"), &out[o], &out[o + 1], seconds, 0.0);
        write_wav(&stems.join(name), &out[o], &out[o + 1])?;
    }
    report(
        "stems/fx.wav",
        &out[FX_OUTPUT],
        &out[FX_OUTPUT + 1],
        seconds,
        0.0,
    );
    write_wav(&stems.join("fx.wav"), &out[FX_OUTPUT], &out[FX_OUTPUT + 1])
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("demo") => {
            let dir = Path::new(args.get(2).map(String::as_str).unwrap_or("."));
            std::fs::create_dir_all(dir)?;
            compare(dir)?;
            four(dir)?;
            generative(dir)?;
            effects(dir)
        }
        Some("controls") => {
            // every automatable parameter: id, name, group, range, default
            for c in front303_core::controls::controls() {
                let range = match c.kind {
                    front303_core::controls::Kind::Knob => "0-127".to_string(),
                    front303_core::controls::Kind::Range { min, max, unit } => {
                        format!("{min}..{max} {unit}")
                    }
                    front303_core::controls::Kind::Int { min, max } => format!("{min}..{max}"),
                    front303_core::controls::Kind::Choice(names) => names.join(" / "),
                    front303_core::controls::Kind::Switch => "OFF / ON".to_string(),
                };
                println!("{}\t{}\t{}\t{}\t{}", c.id, c.name, c.group, range, c.text(c.default));
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: front303 demo [DIR] | front303 controls");
            std::process::exit(2);
        }
    }
}
