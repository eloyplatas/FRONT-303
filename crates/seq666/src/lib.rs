//! SEQ 666: Front 303's four generative sequencers as a plugin with MIDI out and no audio.
//!
//! Each channel plays its own patterns on its own MIDI channel; MIDI notes coming in transpose
//! the channels that follow them (C3 leaves them where they are). Ten memories hold the whole
//! instrument, as in GenderBender P23.

#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::field_reassign_with_default
)]

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use front303_core::{Feedback, GenParams, Pattern, Rng, SLOTS, Scale, generator};
use nice_plug::midi::{Channel, Key, VoiceID};
use nice_plug::prelude::*;

pub mod controls;
pub mod engine;
pub mod gui;
pub mod params;

use controls::{CHANNELS, G};
use engine::{Sequencer, TRANSPOSE_RANGE};
use params::PluginParams;

pub struct Seq666 {
    params: Arc<PluginParams>,
    feedback: Arc<Feedback>,
    seq: Box<Sequencer>,
    values: [f32; controls::COUNT],
    edits: Option<u32>,
    evolutions: [u32; CHANNELS],
    host_was_playing: bool,
    sample_rate: f64,
}

/// The patterns a new instance starts with: a generated line on every pattern of every channel.
pub fn starting_patterns(params: &PluginParams) {
    let mut rng = Rng::new(0x5E0_0666);
    for c in 0..CHANNELS {
        for s in 0..SLOTS {
            let mut g = GenParams::default();
            g.density = [0.8, 0.6, 0.7, 0.5][c];
            g.octaves = [0.35, 0.5, 0.25, 0.6][c];
            let mut p = Pattern::default();
            generator::generate(&mut p, &g, 0, Scale::Minor, &mut rng);
            params.patterns.set(c, s, &p);
        }
    }
}

impl Default for Seq666 {
    fn default() -> Self {
        let params = Arc::new(PluginParams::default());
        starting_patterns(&params);
        Seq666 {
            params,
            feedback: Arc::new(Feedback::default()),
            seq: Box::new(Sequencer::new()),
            values: [0.0; controls::COUNT],
            edits: None,
            evolutions: [0; CHANNELS],
            host_was_playing: false,
            sample_rate: 44_100.0,
        }
    }
}

/// The MIDI note that leaves the sequences untransposed.
const MIDI_C3: i32 = 60;

impl Seq666 {
    fn take_patterns(&mut self) {
        let bank = &self.params.patterns;
        let edits = bank.edits();
        if self.edits != Some(edits) {
            self.edits = Some(edits);
            for c in 0..CHANNELS {
                for s in 0..SLOTS {
                    self.seq.patterns[c][s] = bank.get(c, s);
                }
            }
        }
    }

    fn transport(&mut self, transport: &Transport) {
        let run = self.values[controls::global(G::Run)] >= 0.5;
        let host = transport.playing;
        self.feedback.host.store(host, Relaxed);
        if host {
            if let Some(beats) = transport.pos_beats {
                self.seq.sync(true, beats * 4.0);
            } else if !self.seq.is_playing() {
                self.seq.play();
            }
        } else if run {
            if self.host_was_playing || !self.seq.is_playing() {
                self.seq.play();
            }
        } else {
            self.seq.stop();
        }
        self.host_was_playing = host;
    }

    fn report(&mut self) {
        let fb = &self.feedback;
        fb.playing.store(self.seq.is_playing(), Relaxed);
        fb.set_tempo(self.seq.tempo() as f32);
        for c in 0..CHANNELS {
            let step = self.seq.pattern_step(c).map_or(-1, |s| s as i32);
            fb.steps[c].store(step, Relaxed);
            let active = self.seq.active_pattern(c);
            fb.active[c].store(active as u8, Relaxed);
            fb.raise_peak(c, if self.seq.sounding(c) { 1.0 } else { 0.0 });
            if self.seq.evolutions[c] != self.evolutions[c] {
                self.evolutions[c] = self.seq.evolutions[c];
                self.params
                    .patterns
                    .set_from_audio(c, active, &self.seq.patterns[c][active]);
            }
        }
        fb.transpose.store(self.seq.transpose, Relaxed);
    }
}

fn run_until(
    seq: &mut Sequencer,
    from: usize,
    to: usize,
    rate: f64,
    context: &mut impl ProcessContext<Seq666>,
) {
    if to <= from {
        return;
    }
    seq.process(to - from, rate, |e| {
        let timing = from as u32 + e.timing;
        let channel = Channel::Number(e.channel);
        let key = Key::Number(e.note);
        let event = if e.on {
            NoteEvent::NoteOn {
                timing,
                voice_id: VoiceID::Wildcard,
                channel,
                key,
                velocity: e.velocity as f32 / 127.0,
            }
        } else {
            NoteEvent::NoteOff {
                timing,
                voice_id: VoiceID::Wildcard,
                channel,
                key,
                velocity: 0.0,
            }
        };
        let _ = context.try_send_event(event);
    });
}

impl Plugin for Seq666 {
    const NAME: &'static str = "SEQ 666";
    const VENDOR: &'static str = "Eloy Platas";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();
    type Editor = gui::Editor;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        gui::editor(self.params.clone(), self.feedback.clone())
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.sample_rate = config.sample_rate as f64;
        true
    }

    fn reset(&mut self) {
        self.seq.stop();
        self.host_was_playing = false;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.params.values(&mut self.values);
        controls::apply(&self.values, &mut self.seq.params);
        let transport = context.transport();
        self.seq.params.bpm = match transport.tempo {
            Some(t) if t.is_finite() && t > 0.0 => t as f32,
            _ => self.values[controls::global(G::Tempo)],
        };
        let rate = if transport.sample_rate > 0.0 {
            transport.sample_rate as f64
        } else {
            self.sample_rate
        };
        self.take_patterns();
        self.transport(transport);

        let len = buffer.samples();
        let mut done = 0;
        while let Some(event) = context.next_event() {
            let at = (event.timing() as usize).min(len);
            if let NoteEvent::NoteOn { key, velocity, .. } = event
                && let Some(note) = key.number()
                && velocity > 0.0
            {
                run_until(&mut self.seq, done, at, rate, context);
                done = at;
                self.seq.transpose =
                    (note as i32 - MIDI_C3).clamp(-TRANSPOSE_RANGE, TRANSPOSE_RANGE);
            }
        }
        run_until(&mut self.seq, done, len, rate, context);
        self.report();
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for Seq666 {
    const CLAP_ID: &'static str = "com.eloyplatas.seq666";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Four generative sequencers with MIDI out");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Seq666 {
    /// Never changes: hosts find saved instances by it.
    const VST3_CLASS_ID: [u8; 16] = *b"Seq666EloyPlatas";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nice_export_clap!(Seq666);
nice_export_vst3!(Seq666);
