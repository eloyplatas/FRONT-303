//! The host's view of Front 303: one parameter per entry of the core's control table, and the
//! patterns stored with the host's project.
//!
//! The parameters are built from [`front303_core::controls`] at run time instead of being
//! fields of a struct, so the list a host sees and the list the panel draws come from the same
//! table. `Params` is implemented by hand for that reason.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use front303_core::PatternBank;
use front303_core::controls::{self, Control, Kind};
use front303_core::snapshot::Memories;
use nice_plug::prelude::*;
use nice_plug_egui::EguiEditorState;

/// The editor's size in logical pixels, which is also the panel's design size.
pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 1058;

pub enum HostParam {
    Float(FloatParam),
    Int(IntParam),
}

impl HostParam {
    pub fn value(&self) -> f32 {
        match self {
            HostParam::Float(p) => p.value(),
            HostParam::Int(p) => p.value() as f32,
        }
    }

    pub fn ptr(&self) -> ParamPtr {
        match self {
            HostParam::Float(p) => p.as_ptr(),
            HostParam::Int(p) => p.as_ptr(),
        }
    }
}

fn host_param(c: &Control) -> HostParam {
    let text = {
        let c = c.clone();
        Arc::new(move |v: f32| c.text(v))
    };
    match c.kind {
        Kind::Knob => {
            let range = FloatRange::Linear { min: 0.0, max: 1.0 };
            HostParam::Float(
                FloatParam::new(c.name.clone(), c.default, range)
                    .with_value_to_string(Arc::new(move |v| text(v))),
            )
        }
        Kind::Range { min, max, unit } => {
            let bpm = unit == "BPM";
            let range = FloatRange::Linear { min, max };
            HostParam::Float(
                FloatParam::new(c.name.clone(), c.default, range)
                    .with_step_size(if bpm { 0.1 } else { 1.0 })
                    .with_unit(if bpm { " BPM" } else { " ct" })
                    .with_value_to_string(Arc::new(move |v| {
                        if bpm {
                            format!("{v:.1}")
                        } else {
                            format!("{v:+.0}")
                        }
                    })),
            )
        }
        Kind::Int { .. } | Kind::Choice(_) | Kind::Switch => {
            let range = IntRange::Linear {
                min: c.min() as i32,
                max: c.max() as i32,
            };
            let to_string = text.clone();
            let mut p = IntParam::new(c.name.clone(), c.default.round() as i32, range)
                .with_value_to_string(Arc::new(move |v| to_string(v as f32)));
            if let Kind::Choice(names) = c.kind {
                p = p.with_string_to_value(Arc::new(move |s| {
                    let s = s.trim().to_uppercase();
                    names.iter().position(|n| *n == s).map(|i| i as i32)
                }));
            }
            if matches!(c.kind, Kind::Switch) {
                p = p.with_string_to_value(Arc::new(|s| match s.trim().to_uppercase().as_str() {
                    "ON" | "1" => Some(1),
                    "OFF" | "0" => Some(0),
                    _ => None,
                }));
            }
            HostParam::Int(p)
        }
    }
}

pub struct FrontParams {
    pub controls: Vec<Control>,
    pub params: Vec<HostParam>,
    pub patterns: Arc<PatternBank>,
    /// The ten memories, saved with the project. Only the panel and the host's state calls
    /// touch them, never the audio thread.
    pub memories: Arc<Mutex<Memories>>,
    pub editor_state: Arc<EguiEditorState>,
}

impl Default for FrontParams {
    fn default() -> Self {
        let controls = controls::controls();
        let params = controls.iter().map(host_param).collect();
        FrontParams {
            controls,
            params,
            patterns: Arc::new(PatternBank::default()),
            memories: Arc::new(Mutex::new(Memories::default())),
            editor_state: EguiEditorState::from_size(
                nice_plug::editor::dpi::LogicalSize::new(WIDTH as f32, HEIGHT as f32),
                1.0,
            ),
        }
    }
}

impl FrontParams {
    /// Control `i`'s current value.
    pub fn value(&self, i: usize) -> f32 {
        self.params[i].value()
    }

    /// Every control's value, without allocating.
    pub fn values(&self, out: &mut [f32; controls::COUNT]) {
        for (o, p) in out.iter_mut().zip(&self.params) {
            *o = p.value();
        }
    }
}

const PATTERNS_KEY: &str = "patterns";
const MEMORIES_KEY: &str = "memories";

// SAFETY: every pointer points into `self.params`, a vector that is filled once in
// `Default::default` and never grown, moved out of or shrunk afterwards, so the addresses stay
// valid for as long as the `Arc<FrontParams>` the wrapper holds.
unsafe impl Params for FrontParams {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        self.controls
            .iter()
            .zip(&self.params)
            .map(|(c, p)| (c.id.clone(), p.ptr(), c.group.clone()))
            .collect()
    }

    fn serialize_fields(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert(PATTERNS_KEY.to_string(), self.patterns.serialize());
        if let Ok(memories) = self.memories.lock() {
            m.insert(MEMORIES_KEY.to_string(), memories.to_text());
        }
        m
    }

    fn deserialize_fields(&self, serialized: &BTreeMap<String, String>) {
        if let Some(text) = serialized.get(PATTERNS_KEY) {
            self.patterns.deserialize(text);
        }
        if let Ok(mut memories) = self.memories.lock() {
            // a project from before the memories has none
            *memories = serialized
                .get(MEMORIES_KEY)
                .map(|t| Memories::from_text(t))
                .unwrap_or_default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use front303_core::snapshot::{PATTERNS, Snapshot};
    use front303_core::{Pattern, Step};

    #[test]
    fn memories_are_saved_with_the_project() {
        let a = FrontParams::default();
        let mut patterns = vec![Pattern::default(); PATTERNS];
        patterns[5] = Pattern::from_steps(&[Step::note(3).accent()]);
        let snapshot = Snapshot {
            values: controls::defaults(),
            patterns,
        };
        a.memories.lock().unwrap().slots[4] = Some(snapshot.clone());
        let saved = a.serialize_fields();
        let b = FrontParams::default();
        b.deserialize_fields(&saved);
        assert_eq!(b.memories.lock().unwrap().slots[4], Some(snapshot));
        // a project saved before the memories loads with none
        let mut old = saved.clone();
        old.remove(MEMORIES_KEY);
        b.deserialize_fields(&old);
        assert!(b.memories.lock().unwrap().slots.iter().all(|s| s.is_none()));
    }
}
