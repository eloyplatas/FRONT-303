//! SEQ 666's controls: one table for the host's parameters, the panel and the settings, as in
//! Front 303. The identifiers never change: hosts store automation under them.

use front303_core::controls::{Control, Kind};
use front303_core::params::{GenParams, Rate, SLOTS, Scale};

pub use front303_core::controls::{NOTES, RATES, SCALES};

/// Sequencer channels.
pub const CHANNELS: usize = 4;

/// A channel's controls, in their order within the channel's block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum C {
    On,
    MidiChannel,
    Octave,
    Velocity,
    AccentVelocity,
    Gate,
    Pattern,
    Rate,
    MidiTranspose,
    Density,
    Accents,
    Slides,
    Octaves,
    Evolve,
}

pub const PER_CHANNEL: usize = 14;

/// The controls that are not a channel's, after the channels' blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum G {
    Run,
    Tempo,
    Shuffle,
    Root,
    Scale,
}

const GLOBALS: usize = 5;
pub const COUNT: usize = CHANNELS * PER_CHANNEL + GLOBALS;

pub const fn channel(ch: usize, c: C) -> usize {
    ch * PER_CHANNEL + c as usize
}

pub const fn global(g: G) -> usize {
    CHANNELS * PER_CHANNEL + g as usize
}

/// One sequencer channel's settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChannelParams {
    pub on: bool,
    /// 0..15.
    pub midi_channel: u8,
    /// Octaves from C2 (MIDI note 36) for the pattern's C2.
    pub octave: i8,
    pub velocity: u8,
    pub accent_velocity: u8,
    /// The share of a step a note lasts, 0.05..1.
    pub gate: f32,
    pub pattern: u8,
    pub rate: Rate,
    pub midi_transpose: bool,
    pub generator: GenParams,
}

impl Default for ChannelParams {
    fn default() -> Self {
        ChannelParams {
            on: true,
            midi_channel: 0,
            octave: 0,
            velocity: 96,
            accent_velocity: 127,
            gate: 0.5,
            pattern: 0,
            rate: Rate::Sixteenth,
            midi_transpose: true,
            generator: GenParams::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeqParams {
    pub channels: [ChannelParams; CHANNELS],
    pub bpm: f32,
    pub shuffle: f32,
    pub root: u8,
    pub scale: Scale,
}

impl Default for SeqParams {
    fn default() -> Self {
        SeqParams {
            channels: core::array::from_fn(|c| ChannelParams {
                midi_channel: c as u8,
                ..ChannelParams::default()
            }),
            bpm: 125.0,
            shuffle: 0.0,
            root: 0,
            scale: Scale::Minor,
        }
    }
}

/// Every control, in index order.
pub fn controls() -> Vec<Control> {
    let mut out = Vec::with_capacity(COUNT);
    let d = SeqParams::default();
    for ch in 0..CHANNELS {
        let n = ch + 1;
        let cp = d.channels[ch];
        let mut add = |id: &str, name: &str, kind: Kind, default: f32| {
            out.push(Control {
                id: format!("c{n}_{id}"),
                name: format!("SEQ {n} {name}"),
                group: format!("SEQ {n}"),
                kind,
                default,
            });
        };
        add("on", "On", Kind::Switch, 1.0);
        add(
            "midich",
            "MIDI Channel",
            Kind::Int { min: 1, max: 16 },
            n as f32,
        );
        add("octave", "Octave", Kind::Int { min: -3, max: 3 }, 0.0);
        add(
            "velocity",
            "Velocity",
            Kind::Int { min: 1, max: 127 },
            cp.velocity as f32,
        );
        add(
            "accvel",
            "Accent Velocity",
            Kind::Int { min: 1, max: 127 },
            cp.accent_velocity as f32,
        );
        add("gate", "Gate %", Kind::Int { min: 5, max: 100 }, 50.0);
        add(
            "pattern",
            "Pattern",
            Kind::Int {
                min: 1,
                max: SLOTS as i32,
            },
            1.0,
        );
        add("rate", "Step Rate", Kind::Choice(RATES), 2.0);
        add("miditr", "MIDI Transpose", Kind::Switch, 1.0);
        add("density", "Gen Density", Kind::Knob, cp.generator.density);
        add("accents", "Gen Accents", Kind::Knob, cp.generator.accents);
        add("slides", "Gen Slides", Kind::Knob, cp.generator.slides);
        add("octaves", "Gen Octaves", Kind::Knob, cp.generator.octaves);
        add("evolve", "Gen Evolve", Kind::Knob, 0.0);
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
    debug_assert_eq!(out.len(), COUNT);
    out
}

pub fn defaults() -> Vec<f32> {
    controls().iter().map(|c| c.default).collect()
}

/// Whether a memory or an undo step sets control `i`: everything but RUN.
pub fn restores(i: usize) -> bool {
    i != global(G::Run)
}

/// Writes control values into the settings; the tempo is left to the caller.
pub fn apply(values: &[f32], p: &mut SeqParams) {
    let on = |x: f32| x >= 0.5;
    let int = |x: f32, lo: i32, hi: i32| (x.round() as i32).clamp(lo, hi);
    for ch in 0..CHANNELS {
        let at = |c: C| values[channel(ch, c)];
        let cp = &mut p.channels[ch];
        cp.on = on(at(C::On));
        cp.midi_channel = (int(at(C::MidiChannel), 1, 16) - 1) as u8;
        cp.octave = int(at(C::Octave), -3, 3) as i8;
        cp.velocity = int(at(C::Velocity), 1, 127) as u8;
        cp.accent_velocity = int(at(C::AccentVelocity), 1, 127) as u8;
        cp.gate = int(at(C::Gate), 5, 100) as f32 / 100.0;
        cp.pattern = (int(at(C::Pattern), 1, SLOTS as i32) - 1) as u8;
        cp.rate = Rate::ALL[int(at(C::Rate), 0, Rate::ALL.len() as i32 - 1) as usize];
        cp.midi_transpose = on(at(C::MidiTranspose));
        cp.generator = GenParams {
            density: at(C::Density),
            accents: at(C::Accents),
            slides: at(C::Slides),
            octaves: at(C::Octaves),
            evolve: at(C::Evolve),
        };
    }
    let at = |g: G| values[global(g)];
    p.shuffle = at(G::Shuffle);
    p.root = int(at(G::Root), 0, 11) as u8;
    p.scale = Scale::ALL[int(at(G::Scale), 0, Scale::ALL.len() as i32 - 1) as usize];
}
