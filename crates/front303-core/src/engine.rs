//! The engine: four voices, the mixer, the effects and the transport.
//!
//! The mixer block is ReBirth's (`sub_10013444`, as OpenRB-338's `Frame::block` documents
//! it), with four 303 channels where ReBirth has two 303s and two drum machines: each channel
//! renders 64 samples, runs through its inserts in the order distortion, PCF, compressor,
//! feeds the delay's send bus and is mixed to the master with smoothed pan gains; then the
//! delay returns into the master, the compressor may sit on the master, and the master gain
//! applies. The effects are OpenRB-338's objects, unchanged.
//!
//! Front 303 adds to that: a Setekh mode on the distortion, a Valentine mode on the compressor,
//! tape and interval modes on the delay, a reverb with its own sends, a step rate per voice
//! under the one tempo, MIDI transposition of the sequences, and separate outputs for every
//! voice and for the effect returns.
//!
//! Everything runs at 44.1 kHz in blocks of 64 samples: ReBirth's tables and block-rate
//! controls are built for that rate.

use orb_engine::comp::{self, Compressor};
use orb_engine::delay::{self, Delay};
use orb_engine::dist::{self, Distortion};
use orb_engine::frame::{BLOCK, Block, BlockContext, GUARD, InsertEffect, SendEffect, SongDevice};
use orb_engine::pcf::{self, Pcf};
use orb_engine::process::ProcessState;
use orb_engine::tables::Tables;

use crate::dw::DwVoice;
use crate::fx::intervals::{IntervalSettings, Intervals};
use crate::fx::reverb::ReverbUnit;
use crate::fx::setekh::Setekh;
use crate::fx::tape::{Tape, TapeSettings};
use crate::fx::valentine::{self, Valentine};
use crate::generator::{self, Rng};
use crate::o3::O3Voice;
use crate::params::{
    CompMode, CompRoute, DelayMode, DistMode, Model, Output, PCF_PATTERNS, Params, PcfMode, SLOTS,
    Scale, VOICES,
};
use crate::pattern::{Event, Events, MAX_STEPS, Pattern};
use crate::rb::RbVoice;

/// The engine's sample rate.
pub const SAMPLE_RATE: f64 = 44_100.0;
/// ReBirth's ticks per sixteenth step; the PCF's patterns follow the tick.
const TICKS_PER_STEP: f64 = 48.0;
/// The mixer recomputes its targets every 16 blocks.
const UPDATE_PERIOD: u32 = 16;
/// The meter index's step per block; the effects' meters read their bus there.
const METER_STEP: usize = 23;
/// The gain smoothing of the channels and the master, per block.
const KEEP: f64 = 0.93;
const TAKE: f64 = 0.07;
/// The pan gains ramp in 32 steps of two samples.
const RAMP: f64 = 1.0 / 32.0;
/// The master gain at level 127.
const MASTER_SCALE: f64 = 2.0;
const BPM_MIN: f32 = 20.0;
const BPM_MAX: f32 = 300.0;
/// Steps per bar: a pattern change waits for the bar line.
const BAR: i64 = 16;
/// The most a shuffled step is delayed, as a share of a step.
const SHUFFLE_MAX: f64 = 0.5;
/// A host position this far from the engine's, in steps, moves the engine to it.
const SYNC_TOLERANCE: f64 = 0.05;
/// The seed of the evolve control's generator.
const SEED: u64 = 0x0303_0303;
/// The engine's output channels: the main pair, a pair per voice, and the effect returns.
pub const OUTPUTS: usize = 2 + 2 * VOICES + 2;
/// The first channel of the effect returns' pair.
pub const FX_OUTPUT: usize = 2 + 2 * VOICES;
/// MIDI transposition reaches this far either way, in semitones.
pub const TRANSPOSE_RANGE: i32 = 24;
/// The output pair of voice `v`.
pub const fn voice_output(v: usize) -> usize {
    2 + 2 * v
}

fn st(x: f64) -> f32 {
    x as f32
}

/// Equal-power gains for a ReBirth pan byte.
fn pan_gains(pan: u8) -> (f32, f32) {
    let x = pan.min(127) as f32 / 127.0 * core::f32::consts::FRAC_PI_2;
    (
        x.cos() * core::f32::consts::SQRT_2,
        x.sin() * core::f32::consts::SQRT_2,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Rebirth,
    Dw,
    Open303,
}

fn family(model: Model) -> Family {
    match model {
        Model::Rebirth | Model::RebirthVintage => Family::Rebirth,
        Model::Dw303 => Family::Dw,
        Model::Open303 => Family::Open303,
    }
}

struct VoiceSlot {
    rb: RbVoice,
    dw: DwVoice,
    o3: O3Voice,
    model: Model,
}

/// One voice's place in its own sequence.
#[derive(Clone, Copy)]
struct Sequencer {
    /// The step the voice last got, -1 before the first.
    step: i64,
    /// The running step's gate-off was sent.
    gate_sent: bool,
}

impl Sequencer {
    /// The sequencer placed at `position`, in its own steps, without retriggering a step
    /// already under way.
    fn at(position: f64) -> Self {
        let step = position.floor();
        let within = position - step;
        if within < 1e-6 {
            Sequencer {
                step: step as i64 - 1,
                gate_sent: false,
            }
        } else {
            Sequencer {
                step: step as i64,
                gate_sent: within >= 0.5,
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    target_l: f32,
    target_r: f32,
    gain_l: f32,
    gain_r: f32,
    send: f32,
    dist: bool,
    pcf: bool,
    comp: bool,
}

pub struct Engine {
    pub params: Params,
    /// Each voice's patterns; `params.voices[v].pattern` chooses the one it plays.
    pub patterns: [[Pattern; SLOTS]; VOICES],
    /// How many times the evolve control has rewritten each voice's playing pattern: a
    /// change tells the owner that `patterns` has news for it.
    pub evolutions: [u32; VOICES],
    /// Each channel's peak over the last block, after its inserts and its level.
    pub peaks: [f32; VOICES],
    /// The master's peak over the last block, left and right.
    pub master_peaks: [f32; 2],
    /// The effect returns' peak over the last block.
    pub fx_peak: f32,
    /// Semitones the voices whose MIDI transpose is on are moved by.
    pub transpose: i32,
    /// The pattern each voice is playing, latched at the bar line.
    active: [usize; VOICES],
    rng: Rng,
    tables: Tables,
    voices: [VoiceSlot; VOICES],
    dist: Distortion,
    setekh: [Setekh; VOICES],
    pcf: Pcf,
    comp: Compressor,
    valentine: Valentine,
    delay: Delay,
    tape: Tape,
    intervals: Intervals,
    delay_mode: DelayMode,
    reverb: ReverbUnit,
    /// The modes of the last block: an effect coming in starts from silence.
    last_modes: (DistMode, CompMode, bool),
    channels: [Channel; VOICES],
    send: Block,
    master: f32,
    master_target: f32,
    master_comp: bool,
    counter: u32,
    meter_index: usize,
    playing: bool,
    /// The position in sixteenth steps since play.
    position: f64,
    /// The sixteenth the transport last passed, -1 before the first: bar lines latch the
    /// patterns.
    sixteenth: i64,
    sequencers: [Sequencer; VOICES],
    stop_pending: bool,
    out: [Block; OUTPUTS],
    /// Samples of `out` already handed out.
    out_pos: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let mut e = Engine {
            params: Params::default(),
            patterns: [[Pattern::default(); SLOTS]; VOICES],
            evolutions: [0; VOICES],
            peaks: [0.0; VOICES],
            master_peaks: [0.0; 2],
            fx_peak: 0.0,
            transpose: 0,
            active: [0; VOICES],
            rng: Rng::new(SEED),
            tables: Tables::new(),
            voices: core::array::from_fn(|_| VoiceSlot {
                rb: RbVoice::new(),
                dw: DwVoice::new(),
                o3: O3Voice::new(),
                model: Model::Rebirth,
            }),
            dist: Distortion::new(),
            setekh: [Setekh::new(); VOICES],
            pcf: Pcf::new(),
            comp: Compressor::new(),
            valentine: Valentine::new(),
            delay: Delay::new(ProcessState::new()),
            tape: Tape::new(),
            intervals: Intervals::new(),
            delay_mode: DelayMode::Rebirth,
            reverb: ReverbUnit::new(),
            last_modes: (DistMode::Rebirth, CompMode::Rebirth, false),
            channels: [Channel::default(); VOICES],
            send: [GUARD; BLOCK],
            master: 1.0,
            master_target: 1.0,
            master_comp: false,
            counter: 0,
            meter_index: 0,
            playing: false,
            position: 0.0,
            sixteenth: -1,
            sequencers: [Sequencer {
                step: -1,
                gate_sent: false,
            }; VOICES],
            stop_pending: false,
            out: [[0.0; BLOCK]; OUTPUTS],
            out_pos: BLOCK,
        };
        e.delay.silence();
        e
    }

    /// Starts the patterns from their first step.
    pub fn play(&mut self) {
        self.play_from(0.0);
    }

    /// Starts the patterns at `position`, in sixteenth steps from the start of the song.
    pub fn play_from(&mut self, position: f64) {
        self.playing = true;
        self.locate(position);
    }

    /// Moves to `position` without retriggering a step already under way.
    pub fn locate(&mut self, position: f64) {
        let position = if position.is_finite() { position } else { 0.0 };
        self.position = position;
        self.sixteenth = Sequencer::at(position).step;
        for v in 0..VOICES {
            let rate = self.params.voices[v].rate.sixteenths();
            self.sequencers[v] = Sequencer::at(position / rate);
        }
        self.stop_pending = true;
        self.latch_patterns();
    }

    /// Follows a host's transport: `position` is its position in sixteenth steps at the
    /// start of the audio about to be rendered.
    pub fn sync(&mut self, playing: bool, position: f64) {
        if !playing {
            self.stop();
        } else if !self.playing {
            self.play_from(position);
        } else if (self.position - position).abs() > SYNC_TOLERANCE {
            self.locate(position);
        }
    }

    fn latch_patterns(&mut self) {
        for (a, v) in self.active.iter_mut().zip(&self.params.voices) {
            *a = (v.pattern as usize).min(SLOTS - 1);
        }
    }

    /// The pattern voice `v` is playing.
    pub fn active_pattern(&self, v: usize) -> usize {
        self.active[v]
    }

    /// The step of its pattern voice `v` is on, `None` when stopped.
    pub fn pattern_step(&self, v: usize) -> Option<usize> {
        if !self.playing {
            return None;
        }
        let step = self.sequencers[v].step;
        if step < 0 {
            return None;
        }
        let len = (self.patterns[v][self.active[v]].len as i64).clamp(1, MAX_STEPS as i64);
        Some(step.rem_euclid(len) as usize)
    }

    /// Stops the patterns and closes every gate; tails ring out.
    pub fn stop(&mut self) {
        if self.playing {
            self.playing = false;
            self.stop_pending = true;
        }
    }

    /// Samples already rendered that [`Engine::render`] has not handed out yet.
    pub fn buffered(&self) -> usize {
        BLOCK - self.out_pos
    }

    /// The tempo in use, after its limits.
    pub fn tempo(&self) -> f64 {
        self.bpm()
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// The sixteenth the transport is on, `None` when stopped or before the first.
    pub fn current_step(&self) -> Option<i64> {
        (self.playing && self.sixteenth >= 0).then_some(self.sixteenth)
    }

    fn bpm(&self) -> f64 {
        let b = self.params.bpm;
        (if b.is_finite() {
            b.clamp(BPM_MIN, BPM_MAX)
        } else {
            125.0
        }) as f64
    }

    /// Writes the effects' panels into their devices.
    fn effect_panels(&mut self) {
        let p = &self.params;
        let d = &mut self.dist.live;
        d[dist::ENABLED] = p.dist.on as u8;
        d[dist::AMOUNT] = p.dist.amount.min(127);
        d[dist::SHAPE] = p.dist.shape.min(127);
        let f = &mut self.pcf.live;
        f[pcf::ENABLED] = p.pcf.on as u8;
        f[pcf::FREQUENCY] = p.pcf.frequency.min(127);
        f[pcf::RESONANCE] = p.pcf.resonance.min(127);
        f[pcf::AMOUNT] = p.pcf.amount.min(127);
        f[pcf::WAVE] = p.pcf.pattern.min(PCF_PATTERNS - 1);
        f[pcf::DECAY] = p.pcf.decay.min(127);
        f[pcf::MODE] = (p.pcf.mode == PcfMode::Lowpass) as u8;
        let c = &mut self.comp.live;
        c[comp::ENABLED] = p.comp.on as u8;
        c[comp::AMOUNT] = p.comp.amount.min(127);
        c[comp::THRESHOLD] = p.comp.threshold.min(127);
        let l = &mut self.delay.live;
        l[delay::ENABLED] = (p.delay.on && p.delay.mode == DelayMode::Rebirth) as u8;
        l[delay::STEPS] = p.delay.steps.clamp(1, 32);
        l[delay::MODE] = !p.delay.triplet as u8;
        l[delay::FEEDBACK] = p.delay.feedback.min(127);
        l[delay::PAN] = p.delay.pan.min(127);
    }

    /// The mixer update (`sub_10012C14`): the pan, send and master targets and the routing.
    fn mixer_update(&mut self, ctx_tempo: u32, tick: i32) {
        let t = &self.tables.level;
        let pan = &self.tables.pan;
        let p = &self.params;
        for (c, ch) in self.channels.iter_mut().enumerate() {
            let cp = p.channels[c];
            let lev = if cp.on { cp.level.min(127) } else { 0 } as usize;
            let pg = pan[lev][cp.pan.min(127) as usize];
            ch.target_l = st(GUARD as f64 + pg[0] as f64);
            ch.target_r = st(GUARD as f64 + pg[1] as f64);
            ch.send = st(t[lev] as f64 * t[cp.send.min(127) as usize] as f64 + GUARD as f64);
            ch.dist = cp.dist;
            ch.pcf = p.pcf.channel == Some(c);
            ch.comp = p.comp.route == CompRoute::Channel(c);
        }
        self.master_target = st(MASTER_SCALE * t[p.master.min(127) as usize] as f64 + GUARD as f64);
        self.master_comp = p.comp.route == CompRoute::Master;
        let ctx = BlockContext {
            tables: &self.tables,
            meter_index: self.meter_index,
            tempo: ctx_tempo,
            tick,
            stack_word: None,
        };
        self.dist.update(&ctx);
        self.pcf.update(&ctx);
        self.comp.update(&ctx);
    }

    /// The step a voice plays: its pattern's, in the scale, moved by the MIDI transposition.
    fn step_for(&mut self, v: usize, step: i64, first: bool) -> crate::pattern::Step {
        let (root, scale) = (self.params.root, self.params.scale);
        let vp = self.params.voices[v];
        let g = vp.generator;
        let p = &mut self.patterns[v][self.active[v]];
        let len = (p.len as i64).clamp(1, MAX_STEPS as i64);
        // the pattern came round: the evolve control rewrites some of it
        if !first && step.rem_euclid(len) == 0 && g.evolve > 0.0 {
            generator::mutate(p, &g, g.evolve, root, scale, &mut self.rng);
            self.evolutions[v] = self.evolutions[v].wrapping_add(1);
        }
        let mut s = p.step(step);
        if scale != Scale::Chromatic {
            s.note = generator::quantize(s.note, root, scale);
        }
        if vp.midi_transpose {
            let t = self.transpose.clamp(-TRANSPOSE_RANGE, TRANSPOSE_RANGE);
            s.note = (s.note as i32 + t).clamp(-48, 72) as i8;
        }
        s
    }

    /// The transport over one block: each voice's events, and the tick at the block's start.
    fn advance(&mut self) -> ([Events; VOICES], i32) {
        let mut events = [Events::default(); VOICES];
        if core::mem::take(&mut self.stop_pending) {
            for e in &mut events {
                e.push(0, Event::AllOff);
            }
        }
        let tick = (self.position * TICKS_PER_STEP) as i32;
        if !self.playing {
            self.latch_patterns();
            return (events, tick);
        }
        let inc = self.bpm() * 4.0 / 60.0 / SAMPLE_RATE;
        let shuffle = self.params.shuffle;
        let late = if shuffle.is_finite() {
            shuffle.clamp(0.0, 1.0) as f64 * SHUFFLE_MAX
        } else {
            0.0
        };
        let rates = self.params.voices.map(|v| v.rate.sixteenths());
        for i in 0..BLOCK {
            // the bar line, on the sixteenths, latches the patterns chosen since
            let sixteenth = self.position.floor() as i64;
            if sixteenth != self.sixteenth {
                self.sixteenth = sixteenth;
                if sixteenth.rem_euclid(BAR) == 0 {
                    self.latch_patterns();
                }
            }
            for v in 0..VOICES {
                let position = self.position / rates[v];
                // every second step starts late by the shuffle
                let mut step = position.floor() as i64;
                if step.rem_euclid(2) == 1 && position - (step as f64) < late {
                    step -= 1;
                }
                let start = step as f64 + if step.rem_euclid(2) == 1 { late } else { 0.0 };
                let seq = self.sequencers[v];
                if step > seq.step || step < seq.step - 1 {
                    let first = seq.step < 0;
                    self.sequencers[v] = Sequencer {
                        step,
                        gate_sent: false,
                    };
                    let s = self.step_for(v, step, first);
                    events[v].push(i, Event::Step(s));
                }
                if !self.sequencers[v].gate_sent && position - start >= 0.5 {
                    self.sequencers[v].gate_sent = true;
                    events[v].push(i, Event::GateOff);
                }
            }
            self.position += inc;
        }
        (events, tick)
    }

    /// The delay's time unit, a sixteenth or a triplet eighth, in seconds.
    fn delay_unit(&self) -> f64 {
        let sixteenth = 15.0 / self.bpm();
        if self.params.delay.triplet {
            sixteenth * 4.0 / 3.0
        } else {
            sixteenth
        }
    }

    /// The delay over one block: the send bus in, the return added to `fx`.
    fn run_delay(&mut self, fx: &mut [Block; 2], tempo: u32, tick: i32) {
        let p = self.params.delay;
        if p.mode != self.delay_mode {
            // a delay coming back starts from silence
            match p.mode {
                DelayMode::Tape => self.tape.reset(),
                DelayMode::Intervals => self.intervals.reset(),
                DelayMode::Rebirth => self.delay.silence(),
            }
            self.delay_mode = p.mode;
        }
        let [fl, fr] = fx;
        match p.mode {
            DelayMode::Rebirth => {
                let ctx = BlockContext {
                    tables: &self.tables,
                    meter_index: self.meter_index,
                    tempo,
                    tick,
                    stack_word: None,
                };
                self.delay.process(&mut self.send, fl, fr, &ctx)
            }
            DelayMode::Tape | DelayMode::Intervals if !p.on => self.send.fill(GUARD),
            DelayMode::Tape => {
                let kind = p.kind.min(11) as usize;
                let c = p.c as f64 / 127.0;
                let settings = TapeSettings {
                    time: p.steps.clamp(1, 32) as f64 * self.delay_unit(),
                    feedback: 1.5 * p.feedback.min(127) as f64 / 127.0,
                    wow: (p.a as f64 / 127.0).powi(2),
                    flutter: (p.b as f64 / 127.0).powi(2),
                    scrape: 0.3 * c,
                    noise: 0.01 + 4.0 * c * c,
                    speed: kind / 4,
                    bias: kind % 4,
                };
                let mut echo = [0.0f32; BLOCK];
                self.tape.process(&settings, &self.send, &mut echo);
                let gain = (1.0 / Tape::input_level(settings.speed)) as f32;
                let (gl, gr) = pan_gains(p.pan);
                for i in 0..BLOCK {
                    fl[i] += echo[i] * gain * gl;
                    fr[i] += echo[i] * gain * gr;
                }
                self.send.fill(GUARD);
            }
            DelayMode::Intervals => {
                let settings = IntervalSettings {
                    interval: p.steps.clamp(1, 32) as f64 * self.delay_unit(),
                    pattern: (p.kind % 8) as usize,
                    falloff: 0.6 * p.a as f64 / 127.0,
                    tone: p.b as f64 / 127.0,
                    loop_gain: p.feedback.min(127) as f64 / 127.0,
                    spread: p.c as f64 / 127.0,
                };
                let mut l = [0.0f32; BLOCK];
                let mut r = [0.0f32; BLOCK];
                self.intervals
                    .process(&settings, &self.send, &mut l, &mut r);
                let (gl, gr) = pan_gains(p.pan);
                for i in 0..BLOCK {
                    fl[i] += l[i] * gl;
                    fr[i] += r[i] * gr;
                }
                self.send.fill(GUARD);
            }
        }
    }

    /// One block into `out`.
    fn block(&mut self) {
        self.effect_panels();
        let tempo = (self.bpm() * 1000.0).round() as u32;
        let (events, tick) = self.advance();
        if self.counter.is_multiple_of(UPDATE_PERIOD) {
            self.mixer_update(tempo, tick);
        }
        self.counter = self.counter.wrapping_add(1);
        self.meter_index = (self.meter_index + METER_STEP) & (BLOCK - 1);
        let ctx = BlockContext {
            tables: &self.tables,
            meter_index: self.meter_index,
            tempo,
            tick,
            stack_word: None,
        };
        let modes = (
            self.params.dist.mode,
            self.params.comp.mode,
            self.params.reverb.on,
        );
        if modes != self.last_modes {
            if modes.0 != self.last_modes.0 {
                for s in &mut self.setekh {
                    s.reset();
                }
            }
            if modes.1 != self.last_modes.1 {
                self.valentine.reset();
            }
            if modes.2 && !self.last_modes.2 {
                self.reverb.reset();
            }
            self.last_modes = modes;
        }
        let multi = self.params.output == Output::Multi;
        let valentine_on = self.params.comp.on && self.params.comp.mode == CompMode::Valentine;
        let compress = valentine::compress_db(self.params.comp.amount);
        let makeup = valentine::output_db(self.params.comp.threshold);
        let vp = self.params.comp.valentine;
        let setekh = self.params.dist.mode == DistMode::Setekh;
        let drive = 5.0 * self.params.dist.amount.min(127) as f64 / 127.0;
        let input_db = 24.0 * self.params.dist.shape.min(127) as f64 / 127.0;
        let output_db = 48.0 * self.params.dist.level.min(127) as f64 / 127.0 - 24.0;
        let mut main = [[GUARD; BLOCK]; 2];
        let mut reverb_send = [[0.0f32; BLOCK]; 2];
        let mut bus: Block = [0.0; BLOCK];
        for c in 0..VOICES {
            let p = &self.params.voices[c];
            let slot = &mut self.voices[c];
            if p.model != slot.model {
                // the model the voice leaves starts clean the next time it is chosen
                if family(slot.model) != family(p.model) {
                    slot.rb.reset();
                    slot.dw.reset();
                    slot.o3.reset();
                }
                slot.model = p.model;
            }
            match family(p.model) {
                Family::Dw => slot
                    .dw
                    .render(&mut bus, &events[c], p, self.params.dw_quality),
                Family::Open303 => slot.o3.render(&mut bus, &events[c], p),
                Family::Rebirth => slot.rb.render(&mut bus, &events[c], p, &ctx),
            }
            let ch = &mut self.channels[c];
            if ch.dist {
                if setekh {
                    self.setekh[c].process(&mut bus, drive, input_db, output_db);
                } else {
                    self.dist.process(&mut bus, &ctx);
                }
            }
            if ch.pcf {
                self.pcf.process(&mut bus, &ctx);
                // only patterns past the 55th fault, and the panel cannot select them
                let _ = self.pcf.faulted();
            }
            if ch.comp {
                if valentine_on {
                    self.valentine
                        .process(&vp, compress, makeup, &mut bus, None);
                } else {
                    InsertEffect::process(&mut self.comp, &mut bus, &ctx);
                }
            }
            let top = bus.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            self.peaks[c] = top * ch.gain_l.max(ch.gain_r);
            let sg = ch.send as f64;
            for (s, &b) in self.send.iter_mut().zip(bus.iter()) {
                *s = st(b as f64 * sg + *s as f64);
            }
            let rs = self.params.channels[c].reverb.clamp(0.0, 1.0) as f64;
            let old_l = ch.gain_l as f64;
            let old_r = ch.gain_r as f64;
            let new_l = old_l * KEEP + ch.target_l as f64 * TAKE;
            let new_r = old_r * KEEP + ch.target_r as f64 * TAKE;
            ch.gain_l = st(new_l);
            ch.gain_r = st(new_r);
            let dl = (new_l - old_l) * RAMP;
            let dr = (new_r - old_r) * RAMP;
            let (mut gl, mut gr) = (old_l, old_r);
            let [stem_l, stem_r] = self
                .out
                .get_disjoint_mut([voice_output(c), voice_output(c) + 1])
                .expect("distinct outputs");
            for i in (0..BLOCK).step_by(2) {
                for j in i..i + 2 {
                    let b = bus[j] as f64;
                    let (l, r) = (b * gl, b * gr);
                    stem_l[j] = st(l);
                    stem_r[j] = st(r);
                    main[0][j] = st(l + main[0][j] as f64);
                    main[1][j] = st(r + main[1][j] as f64);
                    reverb_send[0][j] += st(l * rs);
                    reverb_send[1][j] += st(r * rs);
                }
                gl += dl;
                gr += dr;
            }
        }
        // the effect returns
        let mut fx = [[0.0f32; BLOCK]; 2];
        self.run_delay(&mut fx, tempo, tick);
        let ctx = BlockContext {
            tables: &self.tables,
            meter_index: self.meter_index,
            tempo,
            tick,
            stack_word: None,
        };
        let r = self.params.reverb;
        if r.on {
            let ds = r.delay_send.clamp(0.0, 1.0);
            if ds > 0.0 {
                for c in 0..2 {
                    for i in 0..BLOCK {
                        reverb_send[c][i] += fx[c][i] * ds;
                    }
                }
            }
            let [fl, fr] = &mut fx;
            self.reverb
                .process(&r, [&reverb_send[0], &reverb_send[1]], [fl, fr]);
        }
        self.fx_peak = fx.iter().flatten().fold(0.0f32, |m, x| m.max(x.abs()));
        for c in 0..2 {
            for i in 0..BLOCK {
                main[c][i] += fx[c][i];
            }
            self.out[FX_OUTPUT + c] = fx[c];
        }
        let [l, r] = &mut main;
        if self.master_comp && !multi {
            if valentine_on {
                self.valentine
                    .process(&vp, compress, makeup, l, Some(&mut r[..]));
            } else {
                self.comp.process_stereo(l, r, &ctx);
            }
        }
        let m = self.master as f64 * KEEP + self.master_target as f64 * TAKE;
        self.master = st(m);
        for x in l.iter_mut().chain(r.iter_mut()) {
            *x = st(*x as f64 * m);
        }
        if multi {
            // the voices and the returns leave on their own outputs
            l.fill(0.0);
            r.fill(0.0);
        }
        self.master_peaks = [
            l.iter().fold(0.0, |m, x| m.max(x.abs())),
            r.iter().fold(0.0, |m, x| m.max(x.abs())),
        ];
        self.out[0] = *l;
        self.out[1] = *r;
        self.out_pos = 0;
    }

    /// Renders `left.len()` samples of the main output at 44.1 kHz (the two slices have the
    /// same length). The output is not clipped: full scale is ±1.
    pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.render_outputs(&mut [left, right]);
    }

    /// Renders every output channel given, in the order of [`OUTPUTS`]: the main pair, a
    /// pair per voice, the effect returns. All the slices have the same length.
    pub fn render_outputs(&mut self, outputs: &mut [&mut [f32]]) {
        let len = outputs.first().map_or(0, |o| o.len());
        debug_assert!(outputs.iter().all(|o| o.len() == len));
        let channels = outputs.len().min(OUTPUTS);
        let mut done = 0;
        while done < len {
            if self.out_pos == BLOCK {
                self.block();
            }
            let n = (BLOCK - self.out_pos).min(len - done);
            for c in 0..channels {
                outputs[c][done..done + n]
                    .copy_from_slice(&self.out[c][self.out_pos..self.out_pos + n]);
            }
            self.out_pos += n;
            done += n;
        }
    }
}
