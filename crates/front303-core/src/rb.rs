//! The ReBirth voice: OpenRB-338's `Tb303`, driven by Front 303's events and tuning.
//!
//! The device's own sound code is used unchanged. Two things are done around it:
//! - the pitch is set here, so the tune control runs in semitones over two octaves either way
//!   and a fine control is added. ReBirth's pitch table already has 64 entries per semitone
//!   and reaches four octaves either side of C2;
//! - notes come from Front 303's sequencer instead of the device's step function.

use orb_engine::frame::{Block, BlockContext, ChannelDevice, SongDevice};
use orb_engine::tb303::{self, Tb303};

use crate::params::{Model, VoiceParams, Wave, byte, tune};
use crate::pattern::{Event, Events};

/// Entries per semitone of ReBirth's pitch table.
const PER_SEMITONE: f64 = 64.0;
/// The table index of C2 with the tune at its centre.
const ORIGIN: f64 = 3072.0;
/// The table's last index.
const TOP: f64 = 6143.0;
/// The tune byte whose table entry is 0 semitones.
const TUNE_CENTRE: u8 = 0x40;

pub(crate) struct RbVoice {
    tb: Tb303,
    /// The last step's note, semitones from C2.
    note: i32,
}

impl RbVoice {
    pub(crate) fn new() -> Self {
        let mut tb = Tb303::new();
        tb.live[tb303::TUNE] = TUNE_CENTRE;
        RbVoice { tb, note: 0 }
    }

    pub(crate) fn reset(&mut self) {
        SongDevice::reset(&mut self.tb);
    }

    /// The device's pitch value (16 per table index) of the held note at the panel's tuning.
    fn pitch(&self, p: &VoiceParams) -> u32 {
        let (semis, cents) = tune(p);
        let index =
            ORIGIN + (self.note + semis) as f64 * PER_SEMITONE + cents * PER_SEMITONE / 100.0;
        (index.round().clamp(0.0, TOP) as u32) << 4
    }

    fn panel(&mut self, p: &VoiceParams) {
        let live = &mut self.tb.live;
        live[tb303::TUNE] = TUNE_CENTRE;
        live[tb303::CUTOFF] = byte(p.cutoff);
        live[tb303::RESONANCE] = byte(p.resonance);
        live[tb303::ENVMOD] = byte(p.env_mod);
        live[tb303::DECAY] = byte(p.decay);
        live[tb303::ACCENT] = byte(p.accent);
        live[tb303::WAVEFORM] = (p.wave == Wave::Square) as u8;
        self.tb.vintage = p.model == Model::RebirthVintage;
    }

    fn event(&mut self, event: Event, p: &VoiceParams) {
        match event {
            Event::Step(s) => {
                // the device glides into this step when the step before it slid
                let glide = self.tb.slide;
                self.note = s.note as i32;
                self.tb
                    .note_on([0, s.slide as u8, s.accent as u8, 0, 0, s.rest as u8]);
                let pitch = self.pitch(p);
                self.tb.pitch_target = pitch;
                if !glide {
                    self.tb.pitch = pitch;
                }
            }
            Event::GateOff => self.tb.note_off(),
            Event::AllOff => self.tb.silence(),
        }
    }

    /// One block. ReBirth computes a 303's controls once per block, so every event of the
    /// block applies at its start, as in the original.
    pub(crate) fn render(
        &mut self,
        bus: &mut Block,
        events: &Events,
        p: &VoiceParams,
        ctx: &BlockContext,
    ) {
        self.panel(p);
        for (_, e) in events.iter() {
            self.event(e, p);
        }
        // a turn of the tune or fine knob reaches a held note through the device's glide
        self.tb.pitch_target = self.pitch(p);
        ChannelDevice::process(&mut self.tb, bus, ctx);
    }
}
