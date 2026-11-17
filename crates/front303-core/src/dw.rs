//! The DW-303 voice: the DW-303 circuit stages, composed as `dw303-voice` composes them.
//!
//! The signal path and its wiring order are DW-303's (`dw303_voice::Voice::tick_circuit`,
//! `docs/circuit/env.md`); the stages themselves are used unchanged. This copy of the
//! composition exists for one reason: Front 303's tune (two octaves either way) and fine
//! controls are added to the pitch control voltage after the DAC and slide stage, at the
//! oscillator's 1 V/octave input, which `dw303_voice::Voice` does not expose. The panel's
//! own TUNING pot stays at its centre.
//!
//! The circuit runs at four times the engine's 44.1 kHz and is decimated by two half-band
//! stages, as DW-303 does at that host rate.

use dw303_env::{
    Accent, AccentSweep, FilterControl, Meg, OutputStage, TriggerPulse, Vca, VcfSignals, Veg,
};
use dw303_osc::{PitchCv, Vco};
use dw303_vcf::{Quality, Vcf};
use dw303_voice::{FULL_SCALE_VOLTS, VCO_BAND_LIMIT_HZ, calibrated_env_components};
use orb_engine::frame::Block;
use ravet_signal::{ConfiguredHalfBand2x, HalfBandIirDesign};

use crate::SAMPLE_RATE;
use crate::params::{DwQuality, VoiceParams, Wave, knob, tune};
use crate::pattern::{Event, Events};

/// Circuit ticks per engine sample.
const OVERSAMPLING: usize = 4;
/// DW-303's knob slew: the pots have no zipper, so steps are smoothed before the circuit.
const KNOB_SMOOTHING_SECONDS: f64 = 0.005;
/// The TUNING pot's centre.
const TUNING_CENTRE: f64 = 0.5;
/// The VOLUME pot, fully up: the mixer channel sets the level.
const VOLUME: f64 = 1.0;
/// The DAC's semitone index of Front 303's note 0 (C2): its index 0 is C1.
const DAC_C2: i32 = 12;
/// The DAC's highest index (6 bits, code = index + 11).
const DAC_TOP: i32 = 52;
/// Cents added so the model's A2 sits at 110 Hz like the ReBirth model's: DW-303's
/// oscillator is fitted 6.7 cents flat of concert pitch (`docs/circuit/osc.md` in DW-303).
pub(crate) const TUNE_TRIM_CENTS: f64 = 6.7;
/// The gain that brings the model's output to the ReBirth voices' level at the mixer, so a
/// change of model keeps the channel's loudness and drives the distortion alike: +18 dB,
/// which is also the fixed output gain DW-303's own engine applies (`OUTPUT_GAIN_DB`). On a
/// pattern at the same knob positions the two models then sit within about 1 dB RMS of each
/// other, except with the filter wide open (`tests/models.rs`, `levels_match`).
pub(crate) const LEVEL_TRIM: f64 = 7.943_282_347_242_815;

pub(crate) struct DwVoice {
    pitch: PitchCv,
    vco: Vco,
    vcf: Vcf,
    trigger: TriggerPulse,
    meg: Meg,
    veg: Veg,
    accent: Accent,
    sweep: AccentSweep,
    filter_control: FilterControl,
    vca: Vca,
    output: OutputStage,
    decimators: [ConfiguredHalfBand2x; 2],
    quality: DwQuality,
    /// The smoothed knobs: cutoff, resonance, env mod, decay, accent.
    knobs: [f64; 5],
    knob_coefficient: f64,
    /// Semitones from C2 of the sounding note.
    note: i32,
    gate: bool,
    accented: bool,
    /// The sounding note was slid into: the slide switch is closed.
    slid: bool,
    /// The sounding step ties into the next one.
    hold: bool,
    /// A new note begins at the next circuit tick.
    attack: bool,
}

impl DwVoice {
    pub(crate) fn new() -> Self {
        let design =
            HalfBandIirDesign::from_attenuation(0.1, 110.0).expect("valid half-band design");
        let mut v = DwVoice {
            pitch: PitchCv::new(dw303_osc::Components::NOMINAL),
            vco: Vco::new(dw303_osc::Components::NOMINAL),
            vcf: Vcf::new(dw303_vcf::Components::NOMINAL),
            trigger: TriggerPulse::new(dw303_env::Components::NOMINAL),
            meg: Meg::new(dw303_env::Components::NOMINAL),
            veg: Veg::new(dw303_env::Components::NOMINAL),
            accent: Accent::new(dw303_env::Components::NOMINAL),
            sweep: AccentSweep::new(dw303_env::Components::NOMINAL),
            filter_control: FilterControl::new(calibrated_env_components()),
            vca: Vca::new(dw303_env::Components::NOMINAL),
            output: OutputStage::new(dw303_env::Components::NOMINAL),
            decimators: core::array::from_fn(|_| ConfiguredHalfBand2x::new(design, design)),
            quality: DwQuality::Full,
            knobs: [0.5; 5],
            knob_coefficient: 0.0,
            note: 0,
            gate: false,
            accented: false,
            slid: false,
            hold: false,
            attack: false,
        };
        let rate = SAMPLE_RATE * OVERSAMPLING as f64;
        let ok = v.pitch.prepare(rate).is_ok()
            && v.vco.prepare(rate).is_ok()
            && v.vcf.prepare(rate).is_ok()
            && v.trigger.prepare(rate).is_ok()
            && v.meg.prepare(rate).is_ok()
            && v.veg.prepare(rate).is_ok()
            && v.accent.prepare(rate).is_ok()
            && v.sweep.prepare(rate).is_ok()
            && v.filter_control.prepare(rate).is_ok()
            && v.vca.prepare(rate).is_ok()
            && v.output.prepare(rate).is_ok();
        assert!(ok, "the circuit stages accept 176.4 kHz");
        v.vco
            .set_band_limit(Some((0.45 * SAMPLE_RATE).min(VCO_BAND_LIMIT_HZ)));
        v.knob_coefficient = 1.0 - (-1.0 / (KNOB_SMOOTHING_SECONDS * rate)).exp();
        v.vcf.set_quality(Quality::Full);
        v.reset();
        v
    }

    pub(crate) fn reset(&mut self) {
        self.pitch.reset();
        self.vco.reset();
        self.vcf.reset();
        self.trigger.reset();
        self.meg.reset();
        self.veg.reset();
        self.accent.reset();
        self.sweep.reset();
        self.filter_control.reset();
        self.vca.reset();
        self.output.reset();
        for d in &mut self.decimators {
            d.reset();
        }
        self.gate = false;
        self.accented = false;
        self.slid = false;
        self.hold = false;
        self.attack = false;
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Step(s) => {
                // the step before this one tied into it
                let tied = self.hold && self.gate;
                self.note = s.note as i32;
                if s.rest {
                    // the pitch follows a rest's note, as on the ReBirth voice; a tie into a
                    // rest ends at the rest's gate-off
                    self.hold = false;
                    self.slid = tied;
                    return;
                }
                self.accented = s.accent;
                self.slid = tied;
                self.attack = !tied;
                self.gate = true;
                self.hold = s.slide;
            }
            Event::GateOff => {
                if !self.hold {
                    self.gate = false;
                }
            }
            Event::AllOff => {
                self.hold = false;
                self.gate = false;
            }
        }
    }

    /// The pitch control voltage's two parts: the DAC's semitone index, and the volts added
    /// after it (whatever the DAC's range does not reach, the fine control and the trim).
    fn pitch_inputs(&self, p: &VoiceParams) -> (u8, f64) {
        let (semis, cents) = tune(p);
        let index = self.note + DAC_C2 + semis;
        let dac = index.clamp(0, DAC_TOP);
        let extra = (index - dac) as f64 + (cents + TUNE_TRIM_CENTS) / 100.0;
        (dac as u8, extra / 12.0)
    }

    /// One oversampled circuit tick: the OUTPUT jack's voltage.
    fn tick_circuit(&mut self, dac: u8, cv_offset: f64, square: bool, attack: bool) -> f64 {
        let [cutoff, resonance, env_mod, decay, accent] = self.knobs;
        let cv = self.pitch.tick(dac, self.slid) + cv_offset;
        let osc = self.vco.tick(cv, TUNING_CENTRE);
        let waveform = if square { osc.square } else { osc.saw };
        let pulse = self.trigger.tick(attack);
        let meg = self.meg.tick(pulse, self.accented, decay);
        let accent_wiper = self.accent.tick(self.accented, meg, accent);
        let sweep = self.sweep.tick(accent_wiper, resonance);
        let ladder_current = self
            .filter_control
            .ladder_current(cutoff, env_mod, meg, sweep);
        self.veg.set_gate(self.gate);
        let veg_current = self.veg.tick(pulse);
        let vcf = self.vcf.tick(waveform, ladder_current, resonance);
        let vca = self.vca.tick(
            VcfSignals {
                output: vcf.output,
                resonance_tap: vcf.resonance_tap,
            },
            veg_current,
            accent_wiper,
        );
        self.output.tick(vca, VOLUME, 0.0)
    }

    fn smooth(&mut self, target: &[f64; 5]) {
        let k = self.knob_coefficient;
        for (current, &t) in self.knobs.iter_mut().zip(target) {
            let delta = t - *current;
            if delta.abs() < 1.0e-7 {
                *current = t;
            } else {
                *current += delta * k;
            }
        }
    }

    /// One block, with every event at its own sample.
    pub(crate) fn render(
        &mut self,
        bus: &mut Block,
        events: &Events,
        p: &VoiceParams,
        quality: DwQuality,
    ) {
        if quality != self.quality {
            self.quality = quality;
            self.vcf.set_quality(match quality {
                DwQuality::Economy => Quality::Economy,
                DwQuality::Full => Quality::Full,
            });
        }
        let target = [
            knob(p.cutoff),
            knob(p.resonance),
            knob(p.env_mod),
            knob(p.decay),
            knob(p.accent),
        ];
        let square = p.wave == Wave::Square;
        let mut pending = events.iter().peekable();
        for (i, out) in bus.iter_mut().enumerate() {
            while let Some(&(offset, e)) = pending.peek() {
                if offset > i {
                    break;
                }
                self.event(e);
                pending.next();
            }
            let (dac, cv) = self.pitch_inputs(p);
            let mut x = [0.0; OVERSAMPLING];
            for s in &mut x {
                self.smooth(&target);
                let attack = core::mem::take(&mut self.attack);
                *s = self.tick_circuit(dac, cv, square, attack);
            }
            let lower = self.decimators[0].downsample([x[0], x[1]]);
            let upper = self.decimators[0].downsample([x[2], x[3]]);
            let y = self.decimators[1].downsample([lower, upper]) / FULL_SCALE_VOLTS;
            *out = (y * LEVEL_TRIM) as f32;
        }
    }
}
