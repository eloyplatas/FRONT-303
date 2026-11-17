//! Front 303 as a plugin: a VST3 and CLAP instrument with the panel of [`gui`].
//!
//! The audio thread owns the [`Engine`]. Each buffer it reads the host's parameters into the
//! engine's settings, takes the patterns the panel edited from the shared [`PatternBank`],
//! follows the host's transport, renders at 44.1 kHz and resamples to the host's rate. MIDI
//! notes transpose the sequences (C3 leaves them where they are), each at its own sample.
//!
//! The outputs: the main stereo pair, then an auxiliary pair per voice and one for the effect
//! returns. In the STEREO output mode only the main pair carries sound.

#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::field_reassign_with_default
)]

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use front303_core::controls::{self, G};
use front303_core::{
    Engine, Feedback, GenParams, OUTPUTS, Output, Pattern, Rng, SAMPLE_RATE, SLOTS, Scale,
    TRANSPOSE_RANGE, VOICES, generator,
};
use nice_plug::prelude::*;

pub mod gui;
pub mod params;
mod resample;

use params::FrontParams;
use resample::Resampler;

pub struct Front303 {
    params: Arc<FrontParams>,
    feedback: Arc<Feedback>,
    engine: Box<Engine>,
    resampler: Resampler,
    values: [f32; controls::COUNT],
    /// The pattern bank's edit count the engine has.
    edits: Option<u32>,
    evolutions: [u32; VOICES],
    host_was_playing: bool,
}

/// The patterns a new instance starts with: a generated line on every pattern of every voice,
/// in C minor. Voice 1 plays by itself; the others' channels start switched off.
fn starting_patterns(params: &FrontParams) {
    let mut rng = Rng::new(0x0F30_3303);
    for v in 0..VOICES {
        for s in 0..SLOTS {
            let mut g = GenParams::default();
            g.density = [0.8, 0.6, 0.7, 0.5][v];
            g.octaves = [0.35, 0.5, 0.25, 0.6][v];
            let mut p = Pattern::default();
            generator::generate(&mut p, &g, 0, Scale::Minor, &mut rng);
            params.patterns.set(v, s, &p);
        }
    }
}

impl Default for Front303 {
    fn default() -> Self {
        let params = Arc::new(FrontParams::default());
        starting_patterns(&params);
        Front303 {
            params,
            feedback: Arc::new(Feedback::default()),
            engine: Box::new(Engine::new()),
            resampler: Resampler::new(SAMPLE_RATE),
            values: [0.0; controls::COUNT],
            edits: None,
            evolutions: [0; VOICES],
            host_was_playing: false,
        }
    }
}

impl Front303 {
    fn take_patterns(&mut self) {
        let bank = &self.params.patterns;
        let edits = bank.edits();
        if self.edits != Some(edits) {
            self.edits = Some(edits);
            for v in 0..VOICES {
                for s in 0..SLOTS {
                    self.engine.patterns[v][s] = bank.get(v, s);
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
                // what is rendered next plays after what is already rendered
                let ahead = self.engine.buffered() as f64 + self.resampler.buffered();
                let steps_per_sample = self.engine.tempo() * 4.0 / 60.0 / SAMPLE_RATE;
                self.engine
                    .sync(true, beats * 4.0 + ahead * steps_per_sample);
            } else if !self.engine.is_playing() {
                self.engine.play();
            }
        } else if run {
            if self.host_was_playing || !self.engine.is_playing() {
                self.engine.play();
            }
        } else {
            self.engine.stop();
        }
        self.host_was_playing = host;
    }

    fn report(&mut self) {
        let fb = &self.feedback;
        fb.playing.store(self.engine.is_playing(), Relaxed);
        fb.set_tempo(self.engine.tempo() as f32);
        for v in 0..VOICES {
            let step = self.engine.pattern_step(v).map_or(-1, |s| s as i32);
            fb.steps[v].store(step, Relaxed);
            let active = self.engine.active_pattern(v);
            fb.active[v].store(active as u8, Relaxed);
            fb.raise_peak(v, self.engine.peaks[v]);
            // the evolve control rewrote the playing pattern: the panel shows it
            if self.engine.evolutions[v] != self.evolutions[v] {
                self.evolutions[v] = self.engine.evolutions[v];
                self.params
                    .patterns
                    .set_from_audio(v, active, &self.engine.patterns[v][active]);
            }
        }
        fb.raise_peak(VOICES, self.engine.master_peaks[0]);
        fb.raise_peak(VOICES + 1, self.engine.master_peaks[1]);
        fb.raise_peak(VOICES + 2, self.engine.fx_peak);
        fb.transpose.store(self.engine.transpose, Relaxed);
    }

    /// Renders samples `from..to` of the first `k` outputs.
    fn render_into(
        &mut self,
        outputs: &mut [&mut [f32]; OUTPUTS],
        k: usize,
        from: usize,
        to: usize,
    ) {
        if to <= from {
            return;
        }
        let mut segment: [&mut [f32]; OUTPUTS] = Default::default();
        for (s, o) in segment.iter_mut().zip(outputs.iter_mut()).take(k) {
            *s = &mut o[from..to];
        }
        // only the main pair is worth resampling in the stereo mode
        let channels = if self.engine.params.output == Output::Multi {
            k
        } else {
            2
        };
        let engine = &mut self.engine;
        self.resampler
            .process(&mut segment[..channels], |out| engine.render_outputs(out));
        for o in &mut segment[channels..k] {
            o.fill(0.0);
        }
    }
}

/// The MIDI note that leaves the sequences untransposed.
const MIDI_C3: i32 = 60;

/// The names of the auxiliary output pairs.
const AUX_NAMES: &[&str] = &["303-1", "303-2", "303-3", "303-4", "FX Returns"];

impl Plugin for Front303 {
    const NAME: &'static str = "Front 303";
    const VENDOR: &'static str = "Eloy Platas";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            aux_output_ports: &[new_nonzero_u32(2); 5],
            names: PortNames {
                layout: Some("Stereo + 5 stereo outputs"),
                main_input: None,
                main_output: Some("Main"),
                aux_inputs: &[],
                aux_outputs: AUX_NAMES,
            },
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
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
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.resampler = Resampler::new(config.sample_rate as f64);
        context.set_latency_samples(self.resampler.latency());
        true
    }

    fn reset(&mut self) {
        self.engine.stop();
        self.host_was_playing = false;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.params.values(&mut self.values);
        controls::apply(&self.values, &mut self.engine.params);
        let transport = context.transport();
        self.engine.params.bpm = match transport.tempo {
            Some(t) if t.is_finite() && t > 0.0 => t as f32,
            _ => self.values[controls::global(G::Tempo)],
        };
        self.take_patterns();
        self.transport(transport);

        // the main pair and whatever auxiliary pairs the host has
        let len = buffer.samples();
        let mut outputs: [&mut [f32]; OUTPUTS] = Default::default();
        let mut k = 0;
        for c in buffer.as_slice().iter_mut().take(2) {
            outputs[k] = &mut c[..];
            k += 1;
        }
        for bus in aux.outputs.iter_mut() {
            for c in bus.as_slice().iter_mut().take(2) {
                if k < OUTPUTS {
                    outputs[k] = &mut c[..];
                    k += 1;
                }
            }
        }
        // a host that gives fewer channels gets the main pair, or silence
        if k < 2 || outputs[..k].iter().any(|o| o.len() != len) {
            for o in outputs.iter_mut() {
                o.fill(0.0);
            }
            self.report();
            return ProcessStatus::KeepAlive;
        }
        // MIDI notes transpose from their own sample on
        let mut done = 0;
        while let Some(event) = context.next_event() {
            let at = (event.timing() as usize).min(len);
            if let NoteEvent::NoteOn { key, velocity, .. } = event
                && let Some(note) = key.number()
                && velocity > 0.0
            {
                self.render_into(&mut outputs, k, done, at);
                done = at;
                self.engine.transpose =
                    (note as i32 - MIDI_C3).clamp(-TRANSPOSE_RANGE, TRANSPOSE_RANGE);
            }
        }
        self.render_into(&mut outputs, k, done, len);
        self.report();
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for Front303 {
    const CLAP_ID: &'static str = "com.eloyplatas.front303";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Four TB-303 voices with generative sequencers");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for Front303 {
    /// Never changes: hosts find saved instances by it.
    const VST3_CLASS_ID: [u8; 16] = *b"Front303EloyPlts";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nice_export_clap!(Front303);
nice_export_vst3!(Front303);
