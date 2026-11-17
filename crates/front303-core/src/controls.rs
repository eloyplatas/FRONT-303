//! The flat list of controls a host automates and a panel shows: one table, so the plugin's
//! parameters, the panel's widgets and the engine's [`Params`] cannot drift apart.
//!
//! A control's value is a plain number: a knob's 0..=1, an integer in its range, a choice's
//! index, or a switch's 0 or 1.

use crate::params::{
    CompMode, CompRoute, DelayMode, DistMode, DwQuality, FINE_RANGE, Model, Output, PCF_PATTERNS,
    Params, PcfMode, Rate, ReverbMode, SLOTS, Scale, TUNE_RANGE, VOICES, Wave,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// 0..=1.
    Knob,
    /// A range of real numbers, with its unit.
    Range {
        min: f32,
        max: f32,
        unit: &'static str,
    },
    /// A range of whole numbers.
    Int {
        min: i32,
        max: i32,
    },
    /// One of a list.
    Choice(&'static [&'static str]),
    Switch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Control {
    /// The stable identifier a host stores automation under. It never changes.
    pub id: String,
    pub name: String,
    /// The host's parameter group.
    pub group: String,
    pub kind: Kind,
    pub default: f32,
}

impl Control {
    pub fn min(&self) -> f32 {
        match self.kind {
            Kind::Range { min, .. } => min,
            Kind::Int { min, .. } => min as f32,
            _ => 0.0,
        }
    }

    pub fn max(&self) -> f32 {
        match self.kind {
            Kind::Knob | Kind::Switch => 1.0,
            Kind::Range { max, .. } => max,
            Kind::Int { max, .. } => max as f32,
            Kind::Choice(names) => (names.len() - 1) as f32,
        }
    }

    /// Whether the control moves in whole steps.
    pub fn stepped(&self) -> bool {
        !matches!(self.kind, Kind::Knob | Kind::Range { .. })
    }

    /// A value brought into the control's range, and onto a step if it has steps.
    pub fn snap(&self, value: f32) -> f32 {
        let v = if value.is_finite() {
            value
        } else {
            self.default
        };
        let v = v.clamp(self.min(), self.max());
        if self.stepped() { v.round() } else { v }
    }

    pub fn normalize(&self, value: f32) -> f32 {
        let span = self.max() - self.min();
        if span > 0.0 {
            ((value - self.min()) / span).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn denormalize(&self, normalized: f32) -> f32 {
        self.snap(self.min() + normalized.clamp(0.0, 1.0) * (self.max() - self.min()))
    }

    /// The value as a panel or a host shows it.
    pub fn text(&self, value: f32) -> String {
        let v = self.snap(value);
        match self.kind {
            Kind::Knob => format!("{:.0}", v * 127.0),
            Kind::Range { unit, .. } => format!("{v:+.0} {unit}").trim_end().to_string(),
            Kind::Int { min, .. } if min < 0 => format!("{:+}", v as i32),
            Kind::Int { .. } => format!("{}", v as i32),
            Kind::Choice(names) => names[v as usize].to_string(),
            Kind::Switch => if v >= 0.5 { "ON" } else { "OFF" }.to_string(),
        }
    }
}

/// A voice's controls, in their order within the voice's block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum V {
    Model,
    Wave,
    Tune,
    Fine,
    Cutoff,
    Resonance,
    EnvMod,
    Decay,
    Accent,
    Pattern,
    On,
    Level,
    Pan,
    Send,
    Dist,
    Density,
    Accents,
    Slides,
    Octaves,
    Evolve,
    Rate,
    MidiTranspose,
    Reverb,
}

/// Controls per voice.
pub const PER_VOICE: usize = 23;

/// The controls that are not a voice's, in their order after the voices' blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum G {
    Run,
    Tempo,
    Shuffle,
    Root,
    Scale,
    Master,
    Quality,
    DistAmount,
    DistShape,
    PcfChannel,
    PcfFrequency,
    PcfResonance,
    PcfAmount,
    PcfPattern,
    PcfDecay,
    PcfMode,
    CompRoute,
    CompAmount,
    CompThreshold,
    DelayOn,
    DelaySteps,
    DelayTriplet,
    DelayFeedback,
    DelayPan,
    DistMode,
    DistLevel,
    CompMode,
    CompRatio,
    CompAttack,
    CompRelease,
    CompSaturate,
    CompCrush,
    CompMix,
    DelayMode,
    DelayA,
    DelayB,
    DelayC,
    DelayKind,
    ReverbOn,
    ReverbMode,
    ReverbProgram,
    ReverbPredelay,
    ReverbSize,
    ReverbDecay,
    ReverbDamping,
    ReverbModulation,
    ReverbWidth,
    ReverbDelaySend,
    ReverbLevel,
    Output,
}

const GLOBALS: usize = 50;
/// The number of controls.
pub const COUNT: usize = VOICES * PER_VOICE + GLOBALS;

/// The index of voice `v`'s control `c`.
pub const fn voice(v: usize, c: V) -> usize {
    v * PER_VOICE + c as usize
}

/// The index of a control that is not a voice's.
pub const fn global(g: G) -> usize {
    VOICES * PER_VOICE + g as usize
}

pub const MODELS: &[&str] = &["REBIRTH", "VINTAGE", "DW-303", "OPEN303"];
pub const RATES: &[&str] = &[
    "1/32", "1/16T", "1/16", "1/8T", "1/16.", "1/8", "1/4T", "1/8.", "1/4",
];
pub const DIST_MODES: &[&str] = &["REBIRTH", "SETEKH"];
pub const COMP_MODES: &[&str] = &["REBIRTH", "VALENTINE"];
pub const DELAY_MODES: &[&str] = &["REBIRTH", "TAPE", "INTERVALS"];
pub const REVERB_MODES: &[&str] = &["HALL", "EARLY", "AETHER", "CLOUD"];
pub const OUTPUTS: &[&str] = &["STEREO", "MULTI-OUT"];
pub const WAVES: &[&str] = &["SAW", "SQUARE"];
pub const NOTES: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
pub const SCALES: &[&str] = &[
    "CHROMATIC",
    "MAJOR",
    "MINOR",
    "DORIAN",
    "PHRYGIAN",
    "LYDIAN",
    "MIXOLYDIAN",
    "LOCRIAN",
    "HARM MINOR",
    "PHRYG DOM",
    "MINOR PENTA",
    "MAJOR PENTA",
    "BLUES",
    "WHOLE TONE",
];
pub const QUALITIES: &[&str] = &["ECONOMY", "FULL"];
pub const PCF_CHANNELS: &[&str] = &["OFF", "303-1", "303-2", "303-3", "303-4"];
pub const PCF_MODES: &[&str] = &["BANDPASS", "LOWPASS"];
pub const COMP_ROUTES: &[&str] = &["OFF", "MASTER", "303-1", "303-2", "303-3", "303-4"];

const BYTE: Kind = Kind::Int { min: 0, max: 127 };

/// Every control, in index order.
pub fn controls() -> Vec<Control> {
    let mut out = Vec::with_capacity(COUNT);
    let d = Params::default();
    for v in 0..VOICES {
        let n = v + 1;
        let vp = d.voices[v];
        let cp = d.channels[v];
        let mut add = |id: &str, name: &str, kind: Kind, default: f32| {
            out.push(Control {
                id: format!("v{n}_{id}"),
                name: format!("303-{n} {name}"),
                group: format!("303-{n}"),
                kind,
                default,
            });
        };
        add("model", "Model", Kind::Choice(MODELS), 0.0);
        add("wave", "Waveform", Kind::Choice(WAVES), 0.0);
        let t = TUNE_RANGE as i32;
        add("tune", "Tune", Kind::Int { min: -t, max: t }, 0.0);
        add(
            "fine",
            "Fine",
            Kind::Range {
                min: -FINE_RANGE,
                max: FINE_RANGE,
                unit: "ct",
            },
            0.0,
        );
        add("cutoff", "Cutoff", Kind::Knob, vp.cutoff);
        add("reso", "Resonance", Kind::Knob, vp.resonance);
        add("envmod", "Env Mod", Kind::Knob, vp.env_mod);
        add("decay", "Decay", Kind::Knob, vp.decay);
        add("accent", "Accent", Kind::Knob, vp.accent);
        add(
            "pattern",
            "Pattern",
            Kind::Int {
                min: 1,
                max: SLOTS as i32,
            },
            1.0,
        );
        // only the first voice plays when an instance starts
        add("on", "On", Kind::Switch, if v == 0 { 1.0 } else { 0.0 });
        add("level", "Level", BYTE, cp.level as f32);
        add("pan", "Pan", BYTE, cp.pan as f32);
        add("send", "Delay Send", BYTE, cp.send as f32);
        add("dist", "Distortion", Kind::Switch, 0.0);
        add("density", "Gen Density", Kind::Knob, vp.generator.density);
        add("accents", "Gen Accents", Kind::Knob, vp.generator.accents);
        add("slides", "Gen Slides", Kind::Knob, vp.generator.slides);
        add("octaves", "Gen Octaves", Kind::Knob, vp.generator.octaves);
        add("evolve", "Gen Evolve", Kind::Knob, 0.0);
        add("rate", "Step Rate", Kind::Choice(RATES), 2.0);
        add("miditr", "MIDI Transpose", Kind::Switch, 1.0);
        add("reverb", "Reverb Send", Kind::Knob, cp.reverb);
    }
    let mut add = |id: &str, name: &str, group: &str, kind: Kind, default: f32| {
        out.push(Control {
            id: id.to_string(),
            name: name.to_string(),
            group: group.to_string(),
            kind,
            default,
        });
    };
    add("run", "Run", "Transport", Kind::Switch, 0.0);
    add(
        "tempo",
        "Tempo",
        "Transport",
        Kind::Range {
            min: 20.0,
            max: 300.0,
            unit: "BPM",
        },
        d.bpm,
    );
    add("shuffle", "Shuffle", "Transport", Kind::Knob, 0.0);
    add("root", "Root", "Scale", Kind::Choice(NOTES), 0.0);
    add("scale", "Scale", "Scale", Kind::Choice(SCALES), 2.0);
    add("master", "Master", "Mixer", BYTE, d.master as f32);
    add(
        "quality",
        "DW-303 Quality",
        "Mixer",
        Kind::Choice(QUALITIES),
        1.0,
    );
    add(
        "dist_amount",
        "Dist Amount",
        "Distortion",
        BYTE,
        d.dist.amount as f32,
    );
    add(
        "dist_shape",
        "Dist Shape",
        "Distortion",
        BYTE,
        d.dist.shape as f32,
    );
    add(
        "pcf_channel",
        "PCF Channel",
        "PCF",
        Kind::Choice(PCF_CHANNELS),
        0.0,
    );
    add(
        "pcf_freq",
        "PCF Frequency",
        "PCF",
        BYTE,
        d.pcf.frequency as f32,
    );
    add(
        "pcf_reso",
        "PCF Resonance",
        "PCF",
        BYTE,
        d.pcf.resonance as f32,
    );
    add("pcf_amount", "PCF Amount", "PCF", BYTE, d.pcf.amount as f32);
    let patterns = Kind::Int {
        min: 1,
        max: PCF_PATTERNS as i32,
    };
    add(
        "pcf_pattern",
        "PCF Pattern",
        "PCF",
        patterns,
        d.pcf.pattern as f32 + 1.0,
    );
    add("pcf_decay", "PCF Decay", "PCF", BYTE, d.pcf.decay as f32);
    add("pcf_mode", "PCF Mode", "PCF", Kind::Choice(PCF_MODES), 1.0);
    add(
        "comp_route",
        "Comp Route",
        "Compressor",
        Kind::Choice(COMP_ROUTES),
        0.0,
    );
    add(
        "comp_amount",
        "Comp Amount",
        "Compressor",
        BYTE,
        d.comp.amount as f32,
    );
    add(
        "comp_threshold",
        "Comp Threshold",
        "Compressor",
        BYTE,
        d.comp.threshold as f32,
    );
    add("delay_on", "Delay On", "Delay", Kind::Switch, 1.0);
    add(
        "delay_steps",
        "Delay Steps",
        "Delay",
        Kind::Int { min: 1, max: 32 },
        d.delay.steps as f32,
    );
    add("delay_triplet", "Delay Triplet", "Delay", Kind::Switch, 0.0);
    add(
        "delay_feedback",
        "Delay Feedback",
        "Delay",
        BYTE,
        d.delay.feedback as f32,
    );
    add("delay_pan", "Delay Pan", "Delay", BYTE, d.delay.pan as f32);
    add(
        "dist_mode",
        "Dist Mode",
        "Distortion",
        Kind::Choice(DIST_MODES),
        0.0,
    );
    add(
        "dist_level",
        "Dist Level",
        "Distortion",
        BYTE,
        d.dist.level as f32,
    );
    add(
        "comp_mode",
        "Comp Mode",
        "Compressor",
        Kind::Choice(COMP_MODES),
        0.0,
    );
    let v = d.comp.valentine;
    add(
        "comp_ratio",
        "Comp Ratio",
        "Compressor",
        Kind::Int { min: 1, max: 21 },
        v.ratio as f32,
    );
    add(
        "comp_attack",
        "Comp Attack",
        "Compressor",
        BYTE,
        v.attack as f32,
    );
    add(
        "comp_release",
        "Comp Release",
        "Compressor",
        BYTE,
        v.release as f32,
    );
    let ten = Kind::Int { min: 0, max: 10 };
    add(
        "comp_saturate",
        "Comp Saturate",
        "Compressor",
        ten,
        v.saturate as f32,
    );
    add(
        "comp_crush",
        "Comp Crush",
        "Compressor",
        ten,
        v.crush as f32,
    );
    add(
        "comp_mix",
        "Comp Mix",
        "Compressor",
        Kind::Int { min: 0, max: 100 },
        v.mix as f32,
    );
    add(
        "delay_mode",
        "Delay Mode",
        "Delay",
        Kind::Choice(DELAY_MODES),
        0.0,
    );
    add(
        "delay_a",
        "Delay Wow / Falloff",
        "Delay",
        BYTE,
        d.delay.a as f32,
    );
    add(
        "delay_b",
        "Delay Flutter / Tone",
        "Delay",
        BYTE,
        d.delay.b as f32,
    );
    add(
        "delay_c",
        "Delay Noise / Spread",
        "Delay",
        BYTE,
        d.delay.c as f32,
    );
    add(
        "delay_kind",
        "Delay Tape / Pattern",
        "Delay",
        Kind::Int { min: 1, max: 12 },
        d.delay.kind as f32 + 1.0,
    );
    let r = d.reverb;
    add("rev_on", "Reverb On", "Reverb", Kind::Switch, 1.0);
    add(
        "rev_mode",
        "Reverb Mode",
        "Reverb",
        Kind::Choice(REVERB_MODES),
        0.0,
    );
    add(
        "rev_program",
        "Reverb Program",
        "Reverb",
        Kind::Int { min: 1, max: 8 },
        r.program as f32 + 1.0,
    );
    add(
        "rev_predelay",
        "Reverb Predelay",
        "Reverb",
        Kind::Knob,
        r.predelay,
    );
    add("rev_size", "Reverb Size", "Reverb", Kind::Knob, r.size);
    add("rev_decay", "Reverb Decay", "Reverb", Kind::Knob, r.decay);
    add(
        "rev_damping",
        "Reverb Damping",
        "Reverb",
        Kind::Knob,
        r.damping,
    );
    add(
        "rev_mod",
        "Reverb Modulation",
        "Reverb",
        Kind::Knob,
        r.modulation,
    );
    add("rev_width", "Reverb Width", "Reverb", Kind::Knob, r.width);
    add(
        "rev_delay",
        "Reverb Delay Send",
        "Reverb",
        Kind::Knob,
        r.delay_send,
    );
    add("rev_level", "Reverb Level", "Reverb", Kind::Knob, r.level);
    add("output", "Output", "Mixer", Kind::Choice(OUTPUTS), 0.0);
    debug_assert_eq!(out.len(), COUNT);
    out
}

/// Writes control values (one per control, in index order) into the engine's settings.
/// The tempo is left to the caller, who knows whether a host supplies one.
pub fn apply(values: &[f32], p: &mut Params) {
    let on = |x: f32| x >= 0.5;
    let b = |x: f32| x.round().clamp(0.0, 127.0) as u8;
    for v in 0..VOICES {
        let at = |c: V| values[voice(v, c)];
        let vp = &mut p.voices[v];
        vp.model = match at(V::Model).round() as i32 {
            0 => Model::Rebirth,
            1 => Model::RebirthVintage,
            2 => Model::Dw303,
            _ => Model::Open303,
        };
        vp.wave = if on(at(V::Wave)) {
            Wave::Square
        } else {
            Wave::Saw
        };
        vp.tune = at(V::Tune)
            .round()
            .clamp(-(TUNE_RANGE as f32), TUNE_RANGE as f32) as i8;
        vp.fine = at(V::Fine);
        vp.cutoff = at(V::Cutoff);
        vp.resonance = at(V::Resonance);
        vp.env_mod = at(V::EnvMod);
        vp.decay = at(V::Decay);
        vp.accent = at(V::Accent);
        vp.pattern = (at(V::Pattern).round() as i32 - 1).clamp(0, SLOTS as i32 - 1) as u8;
        vp.generator.density = at(V::Density);
        vp.generator.accents = at(V::Accents);
        vp.generator.slides = at(V::Slides);
        vp.generator.octaves = at(V::Octaves);
        vp.generator.evolve = at(V::Evolve);
        vp.rate = Rate::ALL[(at(V::Rate).round().max(0.0) as usize).min(Rate::ALL.len() - 1)];
        vp.midi_transpose = on(at(V::MidiTranspose));
        let cp = &mut p.channels[v];
        cp.on = on(at(V::On));
        cp.level = b(at(V::Level));
        cp.pan = b(at(V::Pan));
        cp.send = b(at(V::Send));
        cp.dist = on(at(V::Dist));
        cp.reverb = at(V::Reverb);
    }
    let at = |g: G| values[global(g)];
    p.shuffle = at(G::Shuffle);
    p.root = (at(G::Root).round() as i32).clamp(0, 11) as u8;
    p.scale = Scale::ALL[(at(G::Scale).round() as usize).min(Scale::ALL.len() - 1)];
    p.master = b(at(G::Master));
    p.dw_quality = if on(at(G::Quality)) {
        DwQuality::Full
    } else {
        DwQuality::Economy
    };
    p.dist.on = true;
    p.dist.amount = b(at(G::DistAmount));
    p.dist.shape = b(at(G::DistShape));
    let channel = at(G::PcfChannel).round() as usize;
    p.pcf.on = true;
    p.pcf.channel = (1..=VOICES).contains(&channel).then(|| channel - 1);
    p.pcf.frequency = b(at(G::PcfFrequency));
    p.pcf.resonance = b(at(G::PcfResonance));
    p.pcf.amount = b(at(G::PcfAmount));
    p.pcf.pattern = (at(G::PcfPattern).round() as i32 - 1).clamp(0, PCF_PATTERNS as i32 - 1) as u8;
    p.pcf.decay = b(at(G::PcfDecay));
    p.pcf.mode = if on(at(G::PcfMode)) {
        PcfMode::Lowpass
    } else {
        PcfMode::Bandpass
    };
    p.comp.on = true;
    p.comp.route = match at(G::CompRoute).round() as usize {
        0 => CompRoute::Off,
        1 => CompRoute::Master,
        c => CompRoute::Channel((c - 2).min(VOICES - 1)),
    };
    p.comp.amount = b(at(G::CompAmount));
    p.comp.threshold = b(at(G::CompThreshold));
    p.delay.on = on(at(G::DelayOn));
    p.delay.steps = (at(G::DelaySteps).round() as i32).clamp(1, 32) as u8;
    p.delay.triplet = on(at(G::DelayTriplet));
    p.delay.feedback = b(at(G::DelayFeedback));
    p.delay.pan = b(at(G::DelayPan));
    p.dist.mode = if on(at(G::DistMode)) {
        DistMode::Setekh
    } else {
        DistMode::Rebirth
    };
    p.dist.level = b(at(G::DistLevel));
    p.comp.mode = if on(at(G::CompMode)) {
        CompMode::Valentine
    } else {
        CompMode::Rebirth
    };
    let int = |x: f32, lo: i32, hi: i32| (x.round() as i32).clamp(lo, hi) as u8;
    let v = &mut p.comp.valentine;
    v.ratio = int(at(G::CompRatio), 1, 21);
    v.attack = b(at(G::CompAttack));
    v.release = b(at(G::CompRelease));
    v.saturate = int(at(G::CompSaturate), 0, 10);
    v.crush = int(at(G::CompCrush), 0, 10);
    v.mix = int(at(G::CompMix), 0, 100);
    p.delay.mode = match at(G::DelayMode).round() as i32 {
        1 => DelayMode::Tape,
        2 => DelayMode::Intervals,
        _ => DelayMode::Rebirth,
    };
    p.delay.a = b(at(G::DelayA));
    p.delay.b = b(at(G::DelayB));
    p.delay.c = b(at(G::DelayC));
    p.delay.kind = int(at(G::DelayKind) - 1.0, 0, 11);
    let r = &mut p.reverb;
    r.on = on(at(G::ReverbOn));
    r.mode = ReverbMode::ALL[(at(G::ReverbMode).round().max(0.0) as usize).min(3)];
    r.program = int(at(G::ReverbProgram) - 1.0, 0, 7);
    r.predelay = at(G::ReverbPredelay);
    r.size = at(G::ReverbSize);
    r.decay = at(G::ReverbDecay);
    r.damping = at(G::ReverbDamping);
    r.modulation = at(G::ReverbModulation);
    r.width = at(G::ReverbWidth);
    r.delay_send = at(G::ReverbDelaySend);
    r.level = at(G::ReverbLevel);
    p.output = if on(at(G::Output)) {
        Output::Multi
    } else {
        Output::Stereo
    };
}

/// Every control's default, in index order.
pub fn defaults() -> Vec<f32> {
    controls().iter().map(|c| c.default).collect()
}
