//! Every setting of Front 303, as plain data the host (a plugin, the CLI, a test) owns.
//!
//! Voice knobs are 0..=1 fractions, because the DW-303 model takes continuous panel
//! rotations; the ReBirth model quantises them to its 0..=127 controller bytes. Mixer and
//! effect settings are the ReBirth engine's own 0..=127 bytes, since its tables are indexed
//! by them.

/// The number of 303 voices.
pub const VOICES: usize = 4;
/// The patterns each voice holds.
pub const SLOTS: usize = 8;

/// The scale the patterns are quantised to and generated in, from the root note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Scale {
    /// No quantisation.
    Chromatic,
    Major,
    #[default]
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    HarmonicMinor,
    PhrygianDominant,
    MinorPentatonic,
    MajorPentatonic,
    Blues,
    WholeTone,
}

impl Scale {
    pub const ALL: [Scale; 14] = [
        Scale::Chromatic,
        Scale::Major,
        Scale::Minor,
        Scale::Dorian,
        Scale::Phrygian,
        Scale::Lydian,
        Scale::Mixolydian,
        Scale::Locrian,
        Scale::HarmonicMinor,
        Scale::PhrygianDominant,
        Scale::MinorPentatonic,
        Scale::MajorPentatonic,
        Scale::Blues,
        Scale::WholeTone,
    ];

    /// The scale's notes, in semitones above the root.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            Scale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Scale::PhrygianDominant => &[0, 1, 4, 5, 7, 8, 10],
            Scale::MinorPentatonic => &[0, 3, 5, 7, 10],
            Scale::MajorPentatonic => &[0, 2, 4, 7, 9],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::WholeTone => &[0, 2, 4, 6, 8, 10],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Scale::Chromatic => "CHROMATIC",
            Scale::Major => "MAJOR",
            Scale::Minor => "MINOR",
            Scale::Dorian => "DORIAN",
            Scale::Phrygian => "PHRYGIAN",
            Scale::Lydian => "LYDIAN",
            Scale::Mixolydian => "MIXOLYDIAN",
            Scale::Locrian => "LOCRIAN",
            Scale::HarmonicMinor => "HARM MINOR",
            Scale::PhrygianDominant => "PHRYG DOM",
            Scale::MinorPentatonic => "MINOR PENTA",
            Scale::MajorPentatonic => "MAJOR PENTA",
            Scale::Blues => "BLUES",
            Scale::WholeTone => "WHOLE TONE",
        }
    }
}

/// How a voice's generator writes patterns. Every value is 0..=1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenParams {
    /// The share of steps that play a note.
    pub density: f32,
    /// How often a note is accented.
    pub accents: f32,
    /// How often a note slides into the next.
    pub slides: f32,
    /// How often a note jumps an octave.
    pub octaves: f32,
    /// The share of steps rewritten each time the pattern comes round; 0 leaves it alone.
    pub evolve: f32,
}

impl Default for GenParams {
    fn default() -> Self {
        GenParams {
            density: 0.75,
            accents: 0.3,
            slides: 0.25,
            octaves: 0.35,
            evolve: 0.0,
        }
    }
}

/// Which synthesis model a voice runs.
/// A sequencer's step length. Every voice runs under the one tempo; a step lasts this many
/// sixteenth notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rate {
    ThirtySecond,
    SixteenthTriplet,
    #[default]
    Sixteenth,
    EighthTriplet,
    SixteenthDotted,
    Eighth,
    QuarterTriplet,
    EighthDotted,
    Quarter,
}

impl Rate {
    pub const ALL: [Rate; 9] = [
        Rate::ThirtySecond,
        Rate::SixteenthTriplet,
        Rate::Sixteenth,
        Rate::EighthTriplet,
        Rate::SixteenthDotted,
        Rate::Eighth,
        Rate::QuarterTriplet,
        Rate::EighthDotted,
        Rate::Quarter,
    ];

    /// Sixteenth notes per step.
    pub fn sixteenths(self) -> f64 {
        match self {
            Rate::ThirtySecond => 0.5,
            Rate::SixteenthTriplet => 2.0 / 3.0,
            Rate::Sixteenth => 1.0,
            Rate::EighthTriplet => 4.0 / 3.0,
            Rate::SixteenthDotted => 1.5,
            Rate::Eighth => 2.0,
            Rate::QuarterTriplet => 8.0 / 3.0,
            Rate::EighthDotted => 3.0,
            Rate::Quarter => 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Model {
    /// The ReBirth RB-338 "2.0 sound" engine (OpenRB-338's reimplementation).
    #[default]
    Rebirth,
    /// The ReBirth RB-338 "Vintage" engine (control every 4 samples, stepped resonance).
    RebirthVintage,
    /// The DW-303 circuit model.
    Dw303,
    /// Robin Schmidt's Open303, as in maddanio's fork.
    Open303,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Wave {
    #[default]
    Saw,
    Square,
}

/// The lowest and highest tune setting, in semitones.
pub const TUNE_RANGE: i8 = 24;
/// The fine control's range, in cents either way.
pub const FINE_RANGE: f32 = 100.0;

/// One 303's panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceParams {
    pub model: Model,
    pub wave: Wave,
    /// Semitones, -24..=24.
    pub tune: i8,
    /// Cents, -100..=100.
    pub fine: f32,
    pub cutoff: f32,
    pub resonance: f32,
    pub env_mod: f32,
    pub decay: f32,
    pub accent: f32,
    /// The pattern the voice plays, 0..SLOTS; a change takes effect at the next bar line.
    pub pattern: u8,
    pub generator: GenParams,
    /// The sequencer's step length.
    pub rate: Rate,
    /// External MIDI notes transpose this voice's sequence.
    pub midi_transpose: bool,
}

impl Default for VoiceParams {
    /// ReBirth's power-on panel: cutoff 20, resonance 90, env mod 20, decay 100, accent 64
    /// (of 127), saw.
    fn default() -> Self {
        VoiceParams {
            model: Model::Rebirth,
            wave: Wave::Saw,
            tune: 0,
            fine: 0.0,
            cutoff: 20.0 / 127.0,
            resonance: 90.0 / 127.0,
            env_mod: 20.0 / 127.0,
            decay: 100.0 / 127.0,
            accent: 64.0 / 127.0,
            pattern: 0,
            generator: GenParams::default(),
            rate: Rate::Sixteenth,
            midi_transpose: true,
        }
    }
}

/// One mixer channel (ReBirth bytes, 0..=127).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChannelParams {
    pub on: bool,
    pub level: u8,
    /// 64 is the centre.
    pub pan: u8,
    /// The delay send.
    pub send: u8,
    /// The channel's distortion switch.
    pub dist: bool,
    /// The reverb send, 0..1.
    pub reverb: f32,
}

impl Default for ChannelParams {
    fn default() -> Self {
        ChannelParams {
            on: true,
            level: 100,
            pan: 64,
            send: 0,
            dist: false,
            reverb: 0.0,
        }
    }
}

/// The distortion: one unit shared by every channel whose switch is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DistParams {
    pub on: bool,
    pub mode: DistMode,
    pub amount: u8,
    /// 0 is a hard clip, 1..=127 the waveshaper curves.
    pub shape: u8,
    /// Setekh's output gain, 0..127 over −24..+24 dB.
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DistMode {
    /// ReBirth's distortion.
    #[default]
    Rebirth,
    /// Full FX Media's Setekh: AMOUNT is its drive, SHAPE its input gain.
    Setekh,
}

impl Default for DistParams {
    fn default() -> Self {
        DistParams {
            on: true,
            amount: 70,
            shape: 0,
            mode: DistMode::Rebirth,
            level: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PcfMode {
    #[default]
    Lowpass,
    Bandpass,
}

/// The highest PCF pattern. ReBirth has 55; its engine reads past them for larger values.
pub const PCF_PATTERNS: u8 = 55;

/// The pattern-controlled filter, on one channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PcfParams {
    pub on: bool,
    /// The channel it sits on, `None` for nowhere.
    pub channel: Option<usize>,
    pub frequency: u8,
    pub resonance: u8,
    pub amount: u8,
    /// 0..=54.
    pub pattern: u8,
    pub decay: u8,
    pub mode: PcfMode,
}

impl Default for PcfParams {
    fn default() -> Self {
        PcfParams {
            on: true,
            channel: None,
            frequency: 70,
            resonance: 64,
            amount: 40,
            pattern: 24,
            decay: 90,
            mode: PcfMode::Lowpass,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CompRoute {
    #[default]
    Off,
    Master,
    Channel(usize),
}

/// The compressor, on one channel or on the master.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompParams {
    pub on: bool,
    pub mode: CompMode,
    pub route: CompRoute,
    pub amount: u8,
    pub threshold: u8,
    pub valentine: ValentineParams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CompMode {
    /// ReBirth's compressor.
    #[default]
    Rebirth,
    /// Tote Bag Labs' Valentine: AMOUNT is its COMPRESS (input gain), THRESHOLD its OUTPUT.
    Valentine,
}

/// Valentine's controls besides COMPRESS and OUTPUT, in its own units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValentineParams {
    /// 1..21, as Valentine's RATIO.
    pub ratio: u8,
    /// 0..127 over 0.02..55 ms.
    pub attack: u8,
    /// 0..127 over 2..1100 ms.
    pub release: u8,
    /// 0 is off; otherwise SATURATE 1..10.
    pub saturate: u8,
    /// 0 is off; otherwise CRUSH 1..10.
    pub crush: u8,
    /// Dry/wet, 0..100.
    pub mix: u8,
}

impl Default for ValentineParams {
    fn default() -> Self {
        // Valentine's defaults: ratio 4, attack 1.3 ms, release 350 ms, saturate 1, mix 100 %
        ValentineParams {
            ratio: 4,
            attack: 67,
            release: 104,
            saturate: 1,
            crush: 0,
            mix: 100,
        }
    }
}

impl Default for CompParams {
    fn default() -> Self {
        CompParams {
            on: true,
            route: CompRoute::Off,
            amount: 70,
            threshold: 64,
            mode: CompMode::Rebirth,
            valentine: ValentineParams::default(),
        }
    }
}

/// The delay, fed by the channels' sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayParams {
    pub on: bool,
    pub mode: DelayMode,
    /// The delay time in steps, 1..=32.
    pub steps: u8,
    /// Steps are triplet eighths instead of sixteenths.
    pub triplet: bool,
    pub feedback: u8,
    pub pan: u8,
    /// TAPE: wow. INTERVALS: falloff.
    pub a: u8,
    /// TAPE: flutter. INTERVALS: tone.
    pub b: u8,
    /// TAPE: noise and scrape. INTERVALS: stereo spread.
    pub c: u8,
    /// TAPE: tape speed and bias, 0..11. INTERVALS: the interval pattern, 0..7.
    pub kind: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DelayMode {
    /// ReBirth's delay.
    #[default]
    Rebirth,
    /// IV-X Delay's tape echo.
    Tape,
    /// Repeats placed on a pattern of intervals, after Delay Intervals.
    Intervals,
}

impl Default for DelayParams {
    fn default() -> Self {
        DelayParams {
            on: true,
            steps: 3,
            triplet: false,
            feedback: 80,
            pan: 64,
            mode: DelayMode::Rebirth,
            a: 30,
            b: 30,
            c: 40,
            kind: 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReverbMode {
    /// Dragonfly Hall.
    #[default]
    Hall,
    /// Dragonfly Early Reflections.
    Early,
    /// Aether.
    Aether,
    /// CloudReverb.
    Cloud,
}

impl ReverbMode {
    pub const ALL: [ReverbMode; 4] = [
        ReverbMode::Hall,
        ReverbMode::Early,
        ReverbMode::Aether,
        ReverbMode::Cloud,
    ];
}

/// The reverb, fed by the channels' reverb sends. Its character knobs leave the program as it
/// is at their centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReverbParams {
    pub on: bool,
    pub mode: ReverbMode,
    /// 0..7.
    pub program: u8,
    /// 0..1 over 0..250 ms, before the engine.
    pub predelay: f32,
    pub size: f32,
    pub decay: f32,
    pub damping: f32,
    pub modulation: f32,
    /// 0..1 over mono..150 % stereo.
    pub width: f32,
    /// The delay's return sent on into the reverb, 0..1.
    pub delay_send: f32,
    /// The reverb's return level, 0..1.
    pub level: f32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        ReverbParams {
            on: true,
            mode: ReverbMode::Hall,
            program: 3,
            predelay: 0.08,
            size: 0.5,
            decay: 0.5,
            damping: 0.5,
            modulation: 0.5,
            width: 0.67,
            delay_send: 0.0,
            level: 0.7,
        }
    }
}

/// Where the voices and the effects go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Output {
    /// Everything mixed on the main stereo output.
    #[default]
    Stereo,
    /// Each voice on its own stereo output after its channel strip, the delay and reverb
    /// returns on another; the main output is silent and the master section is left out.
    Multi,
}

/// The DW-303 model's filter solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DwQuality {
    /// Linearised ladder diodes: cheaper, without the ladder's own distortion.
    Economy,
    #[default]
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub voices: [VoiceParams; VOICES],
    pub channels: [ChannelParams; VOICES],
    pub dist: DistParams,
    pub pcf: PcfParams,
    pub comp: CompParams,
    pub delay: DelayParams,
    pub reverb: ReverbParams,
    /// The master level, 0..=127.
    pub master: u8,
    /// 20..=300.
    pub bpm: f32,
    /// 0..=1: how late every second sixteenth plays.
    pub shuffle: f32,
    /// The root note's pitch class, 0 (C) ..= 11 (B).
    pub root: u8,
    pub scale: Scale,
    pub dw_quality: DwQuality,
    pub output: Output,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            voices: [VoiceParams::default(); VOICES],
            channels: [ChannelParams::default(); VOICES],
            dist: DistParams::default(),
            pcf: PcfParams::default(),
            comp: CompParams::default(),
            delay: DelayParams::default(),
            reverb: ReverbParams::default(),
            master: 100,
            bpm: 125.0,
            shuffle: 0.0,
            root: 0,
            scale: Scale::Chromatic,
            dw_quality: DwQuality::Full,
            output: Output::Stereo,
        }
    }
}

/// A 0..=1 knob as a ReBirth controller byte.
pub(crate) fn byte(x: f32) -> u8 {
    if x.is_finite() {
        (x.clamp(0.0, 1.0) * 127.0).round() as u8
    } else {
        0
    }
}

/// A knob for the circuit model: finite and within the pot's travel.
pub(crate) fn knob(x: f32) -> f64 {
    if x.is_finite() {
        x.clamp(0.0, 1.0) as f64
    } else {
        0.0
    }
}

pub(crate) fn tune(p: &VoiceParams) -> (i32, f64) {
    let semis = p.tune.clamp(-TUNE_RANGE, TUNE_RANGE) as i32;
    let cents = if p.fine.is_finite() {
        p.fine.clamp(-FINE_RANGE, FINE_RANGE) as f64
    } else {
        0.0
    };
    (semis, cents)
}
