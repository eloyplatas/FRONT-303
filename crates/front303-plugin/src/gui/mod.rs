//! Front 303's panel: a rack in the manner of ReBirth 2.0, with a header for the transport and
//! the outputs, two rows of modules (distortion, PCF, compressor and delay; reverb and mixer)
//! and the four 303s below, each with its knobs, its generator, its eight patterns, its step
//! rate and a sixteen-step editor.
//!
//! A bar at the bottom holds the ten memories (as in GenderBender P23: a click or a key 1–0
//! recalls, holding for 0.6 s saves) and UNDO / REDO for everything done on the panel.
//!
//! Everything is painted in design coordinates (1280 × 1058) and scaled to the window.

mod art;

use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use egui::{
    Align2, Color32, Context, FontData, FontDefinitions, FontFamily, Id, Pos2, Rect, Sense, Ui,
    vec2,
};
use front303_core::controls::{self, G, Kind, V, global, voice};
use front303_core::generator::{self, quantize};
use front303_core::snapshot::{History, MEMORIES, Snapshot, restores};
use front303_core::{
    Feedback, GenParams, MAX_STEPS, NOTE_MAX, NOTE_MIN, Pattern, ReverbMode, Rng, SLOTS, Scale,
    VOICES, interval_pattern_name, reverb_program_name,
};
use nice_plug::context::gui::GuiContext;
use nice_plug::editor::dpi::LogicalSize;
use nice_plug::editor::{ResizeHint, SizeConstraints};
use nice_plug_egui::{
    EguiEditor, EguiNiceSettings, NiceEguiApp, RepaintNotifier, create_egui_editor,
};

use crate::params::{FrontParams, HEIGHT, WIDTH};
use art::*;

pub type Editor = EguiEditor<Panel>;

const W: f32 = WIDTH as f32;
const H: f32 = HEIGHT as f32;

/// The voices' colours: red, amber, green and blue, as on the panel's name plates.
const VOICE_COLORS: [u32; VOICES] = [0xff4b2b, 0xffa81c, 0x63d34a, 0x38b6ff];
const ORANGE: u32 = 0xff8a1c;

pub fn editor(params: Arc<FrontParams>, feedback: Arc<Feedback>) -> Option<Editor> {
    let hint = ResizeHint {
        can_resize: true,
        can_resize_horizontally: true,
        can_resize_vertically: true,
        preserve_aspect_ratio: true,
        aspect_ratio_width: WIDTH,
        aspect_ratio_height: HEIGHT,
        size_constraints: SizeConstraints::Logical {
            min_size: Some(LogicalSize::new(W * 0.6, H * 0.6)),
            max_size: Some(LogicalSize::new(W * 2.5, H * 2.5)),
        },
    };
    let settings = EguiNiceSettings::new()
        .with_tile("Front 303")
        .with_resize_hint(hint)
        .with_repaint_poll_interval(Duration::from_millis(16));
    create_egui_editor(
        params.editor_state.clone(),
        RepaintNotifier::new(),
        settings,
        Panel::new(params, feedback),
    )
}

/// A knob or a slider being dragged: its control and its value before snapping.
struct Gesture {
    control: usize,
    value: f32,
}

/// A step's note being dragged.
struct NoteDrag {
    voice: usize,
    step: usize,
    start: i8,
    travel: f32,
}

pub struct Panel {
    params: Arc<FrontParams>,
    feedback: Arc<Feedback>,
    gui: Option<GuiContext>,
    textures: Option<Textures>,
    gesture: Option<Gesture>,
    note_drag: Option<NoteDrag>,
    rng: Rng,
    meters: [f32; VOICES + 3],
    /// A value shown beside the control under the pointer, drawn last.
    bubble: Option<(Pos2, String)>,
    history: RefCell<History>,
    /// A snapshot is being applied: its changes take no checkpoints.
    restoring: Cell<bool>,
    /// The copied pattern, and the voice and slot it came from.
    clipboard: Option<(Pattern, usize, usize)>,
    /// The memory last saved or recalled.
    current_memory: Option<usize>,
    /// When each memory was last saved, for its flash.
    saved_at: [Option<Instant>; MEMORIES],
    /// A memory button or key being held.
    hold: Option<Hold>,
}

/// A press on a memory: released early it recalls, held it saves.
struct Hold {
    slot: usize,
    since: Instant,
    /// The digit key held, or `None` for the mouse.
    key: Option<egui::Key>,
    saved: bool,
}

/// How long a memory is held to save it, as in GenderBender P23.
const HOLD_TO_SAVE: Duration = Duration::from_millis(600);
const DIGITS: [egui::Key; MEMORIES] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
    egui::Key::Num0,
];
/// Checkpoint tags: a control's, a pattern's, and one per discrete action.
const TAG_CONTROL: u64 = 1 << 40;
const TAG_PATTERN: u64 = 2 << 40;

impl Panel {
    fn new(params: Arc<FrontParams>, feedback: Arc<Feedback>) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        Panel {
            params,
            feedback,
            gui: None,
            textures: None,
            gesture: None,
            note_drag: None,
            rng: Rng::new(seed),
            meters: [0.0; VOICES + 3],
            bubble: None,
            history: RefCell::new(History::default()),
            restoring: Cell::new(false),
            clipboard: None,
            current_memory: None,
            saved_at: [None; MEMORIES],
            hold: None,
        }
    }

    /// Every control's value and every pattern, as they are now.
    fn capture(&self) -> Snapshot {
        Snapshot {
            values: (0..self.params.controls.len())
                .map(|i| self.value(i))
                .collect(),
            patterns: self.params.patterns.all(),
        }
    }

    /// Sets the panel to a snapshot: the controls it restores that differ, and the patterns.
    fn apply(&self, s: &Snapshot) {
        self.restoring.set(true);
        for (i, &v) in s.values.iter().enumerate() {
            if restores(i) && (self.value(i) - v).abs() > 1e-6 {
                self.set(i, v);
            }
        }
        let bank = &self.params.patterns;
        for v in 0..VOICES {
            for slot in 0..SLOTS {
                let p = s.pattern(v, slot);
                if bank.get(v, slot) != *p {
                    bank.set(v, slot, p);
                }
            }
        }
        self.restoring.set(false);
    }

    /// Records the state before a change tagged `tag`.
    fn checkpoint(&self, tag: u64) {
        if self.restoring.get() {
            return;
        }
        self.history
            .borrow_mut()
            .checkpoint(tag, Instant::now(), || self.capture());
    }

    /// Records the state before a change of its own, never merged with the one before.
    fn checkpoint_action(&self) {
        self.history.borrow_mut().seal();
        self.checkpoint(u64::MAX);
        self.history.borrow_mut().seal();
    }

    fn undo(&mut self) {
        let present = self.capture();
        let back = self.history.borrow_mut().undo(present);
        if let Some(s) = back {
            self.apply(&s);
        }
    }

    fn redo(&mut self) {
        let present = self.capture();
        let forward = self.history.borrow_mut().redo(present);
        if let Some(s) = forward {
            self.apply(&s);
        }
    }

    fn save_memory(&mut self, m: usize) {
        let snapshot = self.capture();
        if let Ok(mut memories) = self.params.memories.lock() {
            memories.slots[m] = Some(snapshot);
        }
        self.current_memory = Some(m);
        self.saved_at[m] = Some(Instant::now());
    }

    fn recall_memory(&mut self, m: usize) {
        let snapshot = self
            .params
            .memories
            .lock()
            .ok()
            .and_then(|memories| memories.slots[m].clone());
        if let Some(s) = snapshot {
            self.checkpoint_action();
            self.apply(&s);
            self.current_memory = Some(m);
        }
    }

    fn memory_used(&self, m: usize) -> bool {
        self.params
            .memories
            .lock()
            .map(|memories| memories.slots[m].is_some())
            .unwrap_or(false)
    }

    fn value(&self, i: usize) -> f32 {
        self.params.value(i)
    }

    fn control(&self, i: usize) -> &controls::Control {
        &self.params.controls[i]
    }

    fn begin(&self, i: usize) {
        self.checkpoint(TAG_CONTROL | i as u64);
        if let Some(gui) = &self.gui {
            unsafe { gui.raw_begin_set_parameter(self.params.params[i].ptr()) };
        }
    }

    fn send(&self, i: usize, value: f32) {
        if let Some(gui) = &self.gui {
            let c = self.control(i);
            let n = c.normalize(c.snap(value));
            unsafe { gui.raw_set_parameter_normalized(self.params.params[i].ptr(), n) };
        }
    }

    fn end(&self, i: usize) {
        if let Some(gui) = &self.gui {
            unsafe { gui.raw_end_set_parameter(self.params.params[i].ptr()) };
        }
    }

    /// One change, as a complete gesture.
    fn set(&self, i: usize, value: f32) {
        self.begin(i);
        self.send(i, value);
        self.end(i);
    }

    /// The pointer's handling of a continuous or stepped control in `rect`: drag up and down
    /// (with shift for fine moves), turn the wheel, double-click for the default.
    fn drag_control(&mut self, ui: &mut Ui, pen: &Pen, rect: Rect, i: usize, pixels: f32) -> bool {
        let id = key(("control", i));
        let resp = ui.interact(rect, id, Sense::click_and_drag());
        let c = self.control(i).clone();
        let span = c.max() - c.min();
        let fine = ui.input(|inp| inp.modifiers.shift || inp.modifiers.alt);
        if resp.drag_started() {
            self.begin(i);
            self.gesture = Some(Gesture {
                control: i,
                value: self.value(i),
            });
        }
        if resp.dragged()
            && let Some(g) = &mut self.gesture
            && g.control == i
        {
            let dy = -resp.drag_delta().y / pen.scale;
            g.value = (g.value + dy * span / pixels * if fine { 0.1 } else { 1.0 })
                .clamp(c.min(), c.max());
            let v = g.value;
            self.send(i, v);
        }
        if resp.drag_stopped() {
            self.end(i);
            self.gesture = None;
        }
        if resp.double_clicked() {
            self.set(i, c.default);
        }
        if resp.hovered() && !resp.dragged() {
            let scroll = ui.input(|inp| inp.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                let step = if c.stepped() {
                    scroll.signum()
                } else {
                    scroll / 400.0 * span * if fine { 0.1 } else { 1.0 }
                };
                self.set(i, self.value(i) + step);
            }
        }
        let active = resp.hovered() || resp.dragged();
        if active {
            let text = format!("{}  {}", c.name, c.text(self.value(i)));
            self.bubble = Some((rect.center_top(), text));
        }
        active
    }

    /// A click that steps a choice or a number on: the left third goes down, the rest up.
    fn step_control(&mut self, ui: &mut Ui, rect: Rect, i: usize) {
        let resp = ui.interact(rect, key(("step", i)), Sense::click());
        let c = self.control(i).clone();
        let v = self.value(i);
        if resp.clicked() {
            let left = resp
                .interact_pointer_pos()
                .is_some_and(|p| p.x < rect.left() + rect.width() / 3.0);
            let next = if left { v - 1.0 } else { v + 1.0 };
            // choices wrap round; numbers stop at their ends
            let next = match c.kind {
                Kind::Choice(_) if next > c.max() => c.min(),
                Kind::Choice(_) if next < c.min() => c.max(),
                _ => next,
            };
            self.set(i, next);
        }
        if resp.secondary_clicked() {
            self.set(i, c.default);
        }
        if resp.hovered() {
            let scroll = ui.input(|inp| inp.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                self.set(i, v + scroll.signum());
            }
        }
    }

    fn toggle(&mut self, ui: &mut Ui, rect: Rect, i: usize) -> bool {
        let resp = ui.interact(rect, key(("toggle", i)), Sense::click());
        if resp.clicked() {
            self.set(i, if self.value(i) >= 0.5 { 0.0 } else { 1.0 });
        }
        resp.is_pointer_button_down_on()
    }

    fn click(
        &self,
        ui: &mut Ui,
        rect: Rect,
        id: impl std::hash::Hash + std::fmt::Debug,
    ) -> (bool, bool) {
        let resp = ui.interact(rect, key(id), Sense::click());
        (resp.clicked(), resp.is_pointer_button_down_on())
    }

    /// The knob of a 303: black, on the silver panel.
    fn big_knob(
        &mut self,
        ui: &mut Ui,
        pen: &Pen,
        x: f32,
        y: f32,
        r: f32,
        i: usize,
        label: &str,
        accent: Color32,
    ) {
        let c = pen.p(x, y);
        let rect = Rect::from_center_size(c, vec2(r * 2.4, r * 2.4) * pen.scale);
        let active = self.drag_control(ui, pen, rect, i, 220.0);
        let ctl = self.control(i).clone();
        let t = ctl.normalize(self.value(i));
        pen.engraved(
            x,
            y - r - 13.0,
            Align2::CENTER_CENTER,
            label,
            9.5,
            rgb(0x222326),
            rgba(0xffffff, 150),
        );
        let ticks = if matches!(ctl.kind, Kind::Int { min: -24, .. }) {
            5
        } else {
            11
        };
        knob_scale(pen, c, r, ticks, rgba(0x2a2b2e, 220));
        knob(pen, c, r, t, accent, active);
    }

    /// The knob of the dark modules.
    fn mod_knob(
        &mut self,
        ui: &mut Ui,
        pen: &Pen,
        x: f32,
        y: f32,
        r: f32,
        i: usize,
        label: &str,
        accent: Color32,
    ) {
        let c = pen.p(x, y);
        let rect = Rect::from_center_size(c, vec2(r * 2.6, r * 2.6) * pen.scale);
        self.drag_control(ui, pen, rect, i, 200.0);
        let ctl = self.control(i).clone();
        let t = ctl.normalize(self.value(i));
        let bipolar = ctl.id.ends_with("pan");
        value_arc(
            pen,
            c,
            r + 4.0,
            t,
            bipolar,
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 200),
        );
        knob_scale(pen, c, r + 3.0, 11, rgba(0x8a8a8a, 140));
        small_knob(pen, c, r, t, accent);
        pen.text(
            x,
            y + r + 11.0,
            Align2::CENTER_CENTER,
            label,
            BOLD,
            8.0,
            rgb(0xb8b8b8),
        );
    }

    /// A choice shown in a window, stepped by clicks on its arrows or turned with the wheel.
    fn selector(
        &mut self,
        ui: &mut Ui,
        pen: &Pen,
        r: Rect,
        i: usize,
        size: f32,
        color: Color32,
        text: Option<String>,
    ) {
        self.step_control(ui, r, i);
        let c = self.control(i).clone();
        let text = text.unwrap_or_else(|| c.text(self.value(i)));
        lcd(pen, r, &text, size, color);
        let (x, y, w, h) = pen.design(r);
        let tri = |cx: f32, d: f32| {
            vec![
                pen.p(cx + d * 3.0, y + h / 2.0),
                pen.p(cx - d * 2.0, y + h / 2.0 - 3.5),
                pen.p(cx - d * 2.0, y + h / 2.0 + 3.5),
            ]
        };
        let dim = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 140);
        pen.painter.add(egui::Shape::convex_polygon(
            tri(x + 7.0, -1.0),
            dim,
            egui::Stroke::NONE,
        ));
        pen.painter.add(egui::Shape::convex_polygon(
            tri(x + w - 7.0, 1.0),
            dim,
            egui::Stroke::NONE,
        ));
    }

    fn header(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures) {
        let r = pen.r(34.0, 8.0, 1212.0, 62.0);
        dark_plate(pen, t, r, rgb(0x2b2c30), rgb(0x18191b));
        for (x, y) in [(42.0, 16.0), (1238.0, 16.0), (42.0, 62.0), (1238.0, 62.0)] {
            screw(pen, pen.p(x, y));
        }
        // the name
        pen.text(
            58.0,
            40.0,
            Align2::LEFT_CENTER,
            "FRONT",
            BOLD,
            30.0,
            rgba(0x000000, 160),
        );
        pen.text(
            57.0,
            38.5,
            Align2::LEFT_CENTER,
            "FRONT",
            BOLD,
            30.0,
            rgb(0xd9dadc),
        );
        pen.text(
            169.0,
            40.0,
            Align2::LEFT_CENTER,
            "303",
            BOLD,
            30.0,
            rgba(0x000000, 160),
        );
        pen.text(
            168.0,
            38.5,
            Align2::LEFT_CENTER,
            "303",
            BOLD,
            30.0,
            rgb(ORANGE),
        );
        pen.text(
            59.0,
            60.0,
            Align2::LEFT_CENTER,
            "QUAD BASS LINE  ·  REBIRTH / DW-303 / OPEN303 VOICES",
            MEDIUM,
            7.5,
            rgb(0x8c8e93),
        );

        // transport
        let run = self.value(global(G::Run)) >= 0.5;
        let br = pen.r(300.0, 22.0, 58.0, 34.0);
        let pressed = self.toggle(ui, br, global(G::Run));
        button(
            pen,
            br,
            if run { "STOP" } else { "RUN" },
            11.0,
            rgb(0x3a3c40),
            run.then_some(rgb(0x47d65a)),
            pressed,
        );
        let playing = self.feedback.playing.load(Relaxed);
        let host = self.feedback.host.load(Relaxed);
        let beat = self.feedback.steps[0].load(Relaxed).rem_euclid(4) == 0;
        led(
            pen,
            pen.p(374.0, 29.0),
            4.0,
            rgb(0x52ff60),
            if playing && beat {
                1.0
            } else if playing {
                0.35
            } else {
                0.0
            },
        );
        pen.text(
            383.0,
            29.0,
            Align2::LEFT_CENTER,
            "PLAY",
            BOLD,
            7.5,
            rgb(0xa0a2a6),
        );
        led(
            pen,
            pen.p(374.0, 49.0),
            4.0,
            rgb(0x3fb0ff),
            if host { 1.0 } else { 0.0 },
        );
        pen.text(
            383.0,
            49.0,
            Align2::LEFT_CENTER,
            "HOST",
            BOLD,
            7.5,
            rgb(0xa0a2a6),
        );

        pen.text(
            470.0,
            14.0,
            Align2::CENTER_CENTER,
            "TEMPO",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        // the tempo in use: the project's when the host gives one, else the BPM knob's
        let tempo = self.feedback.tempo();
        seven_segment(
            pen,
            pen.r(424.0, 20.0, 92.0, 40.0),
            &format!("{tempo:5.1}"),
            rgb(0xff3b22),
        );
        self.mod_knob(
            ui,
            pen,
            545.0,
            36.0,
            13.0,
            global(G::Tempo),
            "BPM",
            rgb(0xff3b22),
        );
        self.mod_knob(
            ui,
            pen,
            597.0,
            36.0,
            13.0,
            global(G::Shuffle),
            "SHUFFLE",
            rgb(ORANGE),
        );

        pen.text(
            670.0,
            16.0,
            Align2::CENTER_CENTER,
            "ROOT",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        self.selector(
            ui,
            pen,
            pen.r(642.0, 24.0, 56.0, 24.0),
            global(G::Root),
            12.0,
            rgb(0x7dffa0),
            None,
        );
        pen.text(
            780.0,
            16.0,
            Align2::CENTER_CENTER,
            "SCALE",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        self.selector(
            ui,
            pen,
            pen.r(708.0, 24.0, 144.0, 24.0),
            global(G::Scale),
            11.0,
            rgb(0x7dffa0),
            None,
        );
        pen.text(
            904.0,
            16.0,
            Align2::CENTER_CENTER,
            "DW-303 SOLVER",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        self.selector(
            ui,
            pen,
            pen.r(864.0, 24.0, 80.0, 24.0),
            global(G::Quality),
            10.0,
            rgb(0xffc04a),
            None,
        );

        // master
        self.mod_knob(
            ui,
            pen,
            1000.0,
            36.0,
            15.0,
            global(G::Master),
            "MASTER",
            rgb(0xe8e8e8),
        );
        for k in 0..2 {
            meter(
                pen,
                pen.r(1030.0 + k as f32 * 12.0, 16.0, 9.0, 46.0),
                self.meters[VOICES + k],
                12,
            );
        }
        // MIDI transposition and the outputs
        pen.text(
            1095.0,
            16.0,
            Align2::CENTER_CENTER,
            "MIDI TR",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        let tr = self.feedback.transpose.load(Relaxed);
        lcd(
            pen,
            pen.r(1068.0, 24.0, 54.0, 24.0),
            &format!("{tr:+}"),
            12.0,
            rgb(0x7dffa0),
        );
        pen.text(
            1180.0,
            16.0,
            Align2::CENTER_CENTER,
            "OUTPUT",
            BOLD,
            7.5,
            rgb(0x8c8e93),
        );
        self.selector(
            ui,
            pen,
            pen.r(1130.0, 24.0, 100.0, 24.0),
            global(G::Output),
            9.5,
            rgb(0xffc04a),
            None,
        );
        pen.text(
            1180.0,
            57.0,
            Align2::CENTER_CENTER,
            if self.value(global(G::Output)) >= 0.5 {
                "VOICES 3-10 · FX 11/12"
            } else {
                "44.1 kHz ENGINE"
            },
            BOLD,
            6.5,
            rgb(0x6c6e73),
        );
    }

    fn module(&self, pen: &Pen, t: &Textures, x: f32, w: f32, title: &str, accent: Color32) {
        self.module_at(pen, t, x, 78.0, w, title, accent);
    }

    fn module_at(
        &self,
        pen: &Pen,
        t: &Textures,
        x: f32,
        y: f32,
        w: f32,
        title: &str,
        accent: Color32,
    ) {
        let r = pen.r(x, y, w, 154.0);
        dark_plate(pen, t, r, rgb(0x2a2b2f), rgb(0x161719));
        let bar = pen.r(x + 1.0, y + 1.0, w - 2.0, 17.0);
        pen.vgradient(
            bar,
            mix(accent, Color32::BLACK, 0.15),
            mix(accent, Color32::BLACK, 0.45),
        );
        pen.text(
            x + 9.0,
            y + 9.5,
            Align2::LEFT_CENTER,
            title,
            BOLD,
            9.5,
            rgb(0x141414),
        );
        screw(pen, pen.p(x + w - 9.0, y + 9.5));
    }

    fn effects(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures) {
        let label = |x: f32, y: f32, s: &str| {
            pen.text(x, y, Align2::CENTER_CENTER, s, BOLD, 7.5, rgb(0x8c8e93))
        };

        // distortion
        let dst = rgb(0xff6a2b);
        self.module(pen, t, 34.0, 172.0, "DISTORTION", dst);
        self.selector(
            ui,
            pen,
            pen.r(44.0, 104.0, 152.0, 20.0),
            global(G::DistMode),
            9.5,
            dst,
            None,
        );
        let setekh = self.value(global(G::DistMode)) >= 0.5;
        if setekh {
            label(120.0, 134.0, "SETEKH · HIGH SHELF + TANH");
            self.mod_knob(
                ui,
                pen,
                66.0,
                170.0,
                15.0,
                global(G::DistAmount),
                "DRIVE",
                dst,
            );
            self.mod_knob(
                ui,
                pen,
                120.0,
                170.0,
                15.0,
                global(G::DistShape),
                "INPUT",
                dst,
            );
            self.mod_knob(
                ui,
                pen,
                174.0,
                170.0,
                15.0,
                global(G::DistLevel),
                "OUTPUT",
                dst,
            );
        } else {
            let shape = self.value(global(G::DistShape));
            label(
                120.0,
                134.0,
                if shape < 0.5 {
                    "SHAPE 0: HARD CLIP"
                } else {
                    "WAVESHAPER"
                },
            );
            self.mod_knob(
                ui,
                pen,
                84.0,
                170.0,
                17.0,
                global(G::DistAmount),
                "AMOUNT",
                dst,
            );
            self.mod_knob(
                ui,
                pen,
                156.0,
                170.0,
                17.0,
                global(G::DistShape),
                "SHAPE",
                dst,
            );
        }
        label(120.0, 218.0, "ON PER CHANNEL: MIXER DST");

        // PCF
        let pcf = rgb(0xffc21c);
        self.module(
            pen,
            t,
            212.0,
            276.0,
            "PCF  ·  PATTERN CONTROLLED FILTER",
            pcf,
        );
        label(250.0, 104.0, "CHANNEL");
        self.selector(
            ui,
            pen,
            pen.r(220.0, 110.0, 62.0, 20.0),
            global(G::PcfChannel),
            9.5,
            pcf,
            None,
        );
        label(322.0, 104.0, "MODE");
        self.selector(
            ui,
            pen,
            pen.r(288.0, 110.0, 68.0, 20.0),
            global(G::PcfMode),
            9.0,
            pcf,
            None,
        );
        label(420.0, 104.0, "PATTERN");
        let pat = format!("PAT {:02}", self.value(global(G::PcfPattern)) as i32);
        self.selector(
            ui,
            pen,
            pen.r(362.0, 110.0, 116.0, 20.0),
            global(G::PcfPattern),
            9.5,
            pcf,
            Some(pat),
        );
        for (k, (g, name)) in [
            (G::PcfFrequency, "FREQ"),
            (G::PcfResonance, "RESO"),
            (G::PcfAmount, "AMOUNT"),
            (G::PcfDecay, "DECAY"),
        ]
        .into_iter()
        .enumerate()
        {
            self.mod_knob(
                ui,
                pen,
                246.0 + k as f32 * 68.0,
                174.0,
                16.0,
                global(g),
                name,
                pcf,
            );
        }

        // compressor
        let comp = rgb(0x45d0ff);
        self.module(pen, t, 494.0, 330.0, "COMPRESSOR", comp);
        label(560.0, 104.0, "MODE");
        self.selector(
            ui,
            pen,
            pen.r(504.0, 110.0, 112.0, 20.0),
            global(G::CompMode),
            9.0,
            comp,
            None,
        );
        label(716.0, 104.0, "ON");
        self.selector(
            ui,
            pen,
            pen.r(624.0, 110.0, 190.0, 20.0),
            global(G::CompRoute),
            9.5,
            comp,
            None,
        );
        if self.value(global(G::CompMode)) >= 0.5 {
            let knobs = [
                (G::CompAmount, "COMPRESS"),
                (G::CompRatio, "RATIO"),
                (G::CompAttack, "ATTACK"),
                (G::CompRelease, "RELEASE"),
                (G::CompSaturate, "SATURATE"),
                (G::CompCrush, "CRUSH"),
                (G::CompMix, "MIX"),
                (G::CompThreshold, "OUTPUT"),
            ];
            for (k, (g, name)) in knobs.into_iter().enumerate() {
                let kx = 530.0 + (k % 4) as f32 * 80.0;
                let ky = 156.0 + (k / 4) as f32 * 46.0;
                self.mod_knob(ui, pen, kx, ky, 11.0, global(g), name, comp);
            }
        } else {
            self.mod_knob(
                ui,
                pen,
                600.0,
                174.0,
                18.0,
                global(G::CompAmount),
                "AMOUNT",
                comp,
            );
            self.mod_knob(
                ui,
                pen,
                718.0,
                174.0,
                18.0,
                global(G::CompThreshold),
                "THRESHOLD",
                comp,
            );
        }

        // delay
        let dly = rgb(0x7dff8a);
        self.module(pen, t, 830.0, 416.0, "DELAY", dly);
        let on = self.value(global(G::DelayOn)) >= 0.5;
        let br = pen.r(840.0, 110.0, 40.0, 20.0);
        let p = self.toggle(ui, br, global(G::DelayOn));
        button(pen, br, "ON", 9.0, rgb(0x3a3c40), on.then_some(dly), p);
        label(926.0, 104.0, "MODE");
        self.selector(
            ui,
            pen,
            pen.r(886.0, 110.0, 82.0, 20.0),
            global(G::DelayMode),
            8.5,
            dly,
            None,
        );
        let trip = self.value(global(G::DelayTriplet)) >= 0.5;
        let br = pen.r(974.0, 110.0, 58.0, 20.0);
        let p = self.toggle(ui, br, global(G::DelayTriplet));
        button(
            pen,
            br,
            "TRIPLET",
            8.0,
            rgb(0x3a3c40),
            trip.then_some(dly),
            p,
        );
        let mode = self.value(global(G::DelayMode)).round() as i32;
        label(1080.0, 104.0, if mode == 2 { "INTERVAL" } else { "TIME" });
        let steps = self.value(global(G::DelaySteps)) as i32;
        let text = format!("{steps} {}", if trip { "× 1/8T" } else { "× 1/16" });
        self.selector(
            ui,
            pen,
            pen.r(1038.0, 110.0, 84.0, 20.0),
            global(G::DelaySteps),
            9.0,
            dly,
            Some(text),
        );
        let kind = self.value(global(G::DelayKind)).round() as usize;
        match mode {
            1 => {
                label(1182.0, 104.0, "TAPE");
                let k = kind.clamp(1, 12) - 1;
                let speed = ["15 IPS", "7.5 IPS", "3.75 IPS"][k / 4];
                let text = format!("{speed} B{}", k % 4 + 2);
                self.selector(
                    ui,
                    pen,
                    pen.r(1128.0, 110.0, 108.0, 20.0),
                    global(G::DelayKind),
                    8.5,
                    dly,
                    Some(text),
                );
            }
            2 => {
                label(1182.0, 104.0, "PATTERN");
                let text = interval_pattern_name((kind.max(1) - 1) % 8).to_string();
                self.selector(
                    ui,
                    pen,
                    pen.r(1128.0, 110.0, 108.0, 20.0),
                    global(G::DelayKind),
                    8.5,
                    dly,
                    Some(text),
                );
            }
            _ => label(1182.0, 120.0, "REBIRTH ECHO"),
        }
        let names: [&str; 3] = match mode {
            1 => ["WOW", "FLUTTER", "NOISE"],
            2 => ["FALLOFF", "TONE", "SPREAD"],
            _ => ["", "", ""],
        };
        let fb_name = if mode == 2 { "LOOP" } else { "FEEDBACK" };
        if mode == 0 {
            self.mod_knob(
                ui,
                pen,
                940.0,
                174.0,
                18.0,
                global(G::DelayFeedback),
                fb_name,
                dly,
            );
            self.mod_knob(
                ui,
                pen,
                1100.0,
                174.0,
                18.0,
                global(G::DelayPan),
                "PAN",
                dly,
            );
        } else {
            let knobs = [
                (G::DelayFeedback, fb_name),
                (G::DelayPan, "PAN"),
                (G::DelayA, names[0]),
                (G::DelayB, names[1]),
                (G::DelayC, names[2]),
            ];
            for (k, (g, name)) in knobs.into_iter().enumerate() {
                self.mod_knob(
                    ui,
                    pen,
                    878.0 + k as f32 * 80.0,
                    176.0,
                    15.0,
                    global(g),
                    name,
                    dly,
                );
            }
        }
    }

    fn reverb(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures) {
        let y = 238.0;
        let rev = rgb(0xc58bff);
        self.module_at(pen, t, 34.0, y, 600.0, "REVERB", rev);
        let label = |x: f32, yy: f32, s: &str| {
            pen.text(x, yy, Align2::CENTER_CENTER, s, BOLD, 7.5, rgb(0x8c8e93))
        };
        let on = self.value(global(G::ReverbOn)) >= 0.5;
        let br = pen.r(44.0, y + 32.0, 40.0, 20.0);
        let p = self.toggle(ui, br, global(G::ReverbOn));
        button(pen, br, "ON", 9.0, rgb(0x3a3c40), on.then_some(rev), p);
        label(132.0, y + 26.0, "ENGINE");
        self.selector(
            ui,
            pen,
            pen.r(90.0, y + 32.0, 84.0, 20.0),
            global(G::ReverbMode),
            9.0,
            rev,
            None,
        );
        let mode = ReverbMode::ALL[(self.value(global(G::ReverbMode)).round() as usize).min(3)];
        let program = (self.value(global(G::ReverbProgram)).round() as usize).clamp(1, 8) - 1;
        let source = match mode {
            ReverbMode::Hall => "DRAGONFLY HALL",
            ReverbMode::Early => "DRAGONFLY EARLY REFLECTIONS",
            ReverbMode::Aether => "AETHER",
            ReverbMode::Cloud => "CLOUDREVERB",
        };
        label(330.0, y + 26.0, &format!("PROGRAM  ·  {source}"));
        let text = format!("{}  {}", program + 1, reverb_program_name(mode, program));
        self.selector(
            ui,
            pen,
            pen.r(182.0, y + 32.0, 296.0, 20.0),
            global(G::ReverbProgram),
            9.5,
            rev,
            Some(text),
        );
        label(560.0, y + 26.0, "RETURN");
        meter(
            pen,
            pen.r(500.0, y + 33.0, 120.0, 8.0),
            self.meters[VOICES + 2],
            16,
        );
        let knobs = [
            (G::ReverbPredelay, "PREDELAY"),
            (G::ReverbSize, "SIZE"),
            (G::ReverbDecay, "DECAY"),
            (G::ReverbDamping, "DAMPING"),
            (G::ReverbModulation, "MOD"),
            (G::ReverbWidth, "WIDTH"),
            (G::ReverbDelaySend, "DLY → REV"),
            (G::ReverbLevel, "LEVEL"),
        ];
        for (k, (g, name)) in knobs.into_iter().enumerate() {
            // the early reflections have no decay or modulation to shape
            let idle =
                mode == ReverbMode::Early && matches!(g, G::ReverbDecay | G::ReverbModulation);
            let col = if idle { rgb(0x5a5a62) } else { rev };
            self.mod_knob(
                ui,
                pen,
                76.0 + k as f32 * 72.0,
                y + 98.0,
                17.0,
                global(g),
                name,
                col,
            );
        }
        pen.text(
            334.0,
            y + 142.0,
            Align2::CENTER_CENTER,
            "SIZE · DECAY · DAMPING · MOD: CENTRE = AS THE PROGRAM",
            MEDIUM,
            7.0,
            rgb(0x6c6e73),
        );
    }

    fn mixer(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures) {
        let y = 238.0;
        let x = 640.0;
        self.module_at(pen, t, x, y, 606.0, "MIXER", rgb(0xd0d2d6));
        let multi = self.value(global(G::Output)) >= 0.5;
        for v in 0..VOICES {
            let x0 = x + 10.0 + v as f32 * 149.0;
            let col = rgb(VOICE_COLORS[v]);
            if v > 0 {
                pen.line(
                    pen.p(x0 - 5.0, y + 24.0),
                    pen.p(x0 - 5.0, y + 148.0),
                    1.0,
                    rgba(0x000000, 160),
                );
                pen.line(
                    pen.p(x0 - 4.0, y + 24.0),
                    pen.p(x0 - 4.0, y + 148.0),
                    1.0,
                    rgba(0xffffff, 18),
                );
            }
            pen.text(
                x0 + 8.0,
                y + 30.0,
                Align2::LEFT_CENTER,
                &format!("303-{}", v + 1),
                BOLD,
                9.0,
                col,
            );
            if multi {
                pen.text(
                    x0 + 136.0,
                    y + 30.0,
                    Align2::RIGHT_CENTER,
                    &format!("OUT {}/{}", 3 + 2 * v, 4 + 2 * v),
                    BOLD,
                    7.0,
                    rgb(0xffc04a),
                );
            }
            self.mod_knob(
                ui,
                pen,
                x0 + 30.0,
                y + 64.0,
                15.0,
                voice(v, V::Level),
                "LEVEL",
                col,
            );
            meter(
                pen,
                pen.r(x0 + 56.0, y + 44.0, 10.0, 46.0),
                self.meters[v],
                9,
            );
            self.mod_knob(
                ui,
                pen,
                x0 + 92.0,
                y + 64.0,
                11.0,
                voice(v, V::Pan),
                "PAN",
                col,
            );
            self.mod_knob(
                ui,
                pen,
                x0 + 22.0,
                y + 116.0,
                10.0,
                voice(v, V::Send),
                "DLY",
                rgb(0x7dff8a),
            );
            self.mod_knob(
                ui,
                pen,
                x0 + 62.0,
                y + 116.0,
                10.0,
                voice(v, V::Reverb),
                "REV",
                rgb(0xc58bff),
            );
            let on = self.value(voice(v, V::On)) >= 0.5;
            let br = pen.r(x0 + 92.0, y + 98.0, 40.0, 17.0);
            let p = self.toggle(ui, br, voice(v, V::On));
            button(pen, br, "ON", 8.0, rgb(0x3a3c40), on.then_some(col), p);
            let dist = self.value(voice(v, V::Dist)) >= 0.5;
            let br = pen.r(x0 + 92.0, y + 120.0, 40.0, 17.0);
            let p = self.toggle(ui, br, voice(v, V::Dist));
            button(
                pen,
                br,
                "DST",
                8.0,
                rgb(0x3a3c40),
                dist.then_some(rgb(0xff6a2b)),
                p,
            );
        }
    }

    /// The pattern voice `v` edits: the one its PATTERN control selects.
    fn slot(&self, v: usize) -> usize {
        (self.value(voice(v, V::Pattern)).round() as usize).clamp(1, SLOTS) - 1
    }

    fn gen_params(&self, v: usize) -> GenParams {
        GenParams {
            density: self.value(voice(v, V::Density)),
            accents: self.value(voice(v, V::Accents)),
            slides: self.value(voice(v, V::Slides)),
            octaves: self.value(voice(v, V::Octaves)),
            evolve: self.value(voice(v, V::Evolve)),
        }
    }

    fn scale(&self) -> (u8, Scale) {
        let root = self.value(global(G::Root)).round() as u8;
        let scale =
            Scale::ALL[(self.value(global(G::Scale)).round() as usize).min(Scale::ALL.len() - 1)];
        (root, scale)
    }

    /// `note` moved `by` notes of the scale.
    fn move_in_scale(&self, note: i8, by: i32) -> i8 {
        let (root, scale) = self.scale();
        let mut n = quantize(note, root, scale) as i32;
        let dir = by.signum();
        for _ in 0..by.abs() {
            let mut m = n + dir;
            while (NOTE_MIN as i32..=NOTE_MAX as i32).contains(&m)
                && quantize(m as i8, root, scale) as i32 != m
            {
                m += dir;
            }
            if !(NOTE_MIN as i32..=NOTE_MAX as i32).contains(&m) {
                break;
            }
            n = m;
        }
        n as i8
    }

    /// A change to the pattern voice `v` edits. Changes in a row (a note dragged or turned)
    /// share one undo step.
    fn edit(&self, v: usize, f: impl FnOnce(&mut Pattern)) {
        let slot = self.slot(v);
        self.checkpoint(TAG_PATTERN | (v * SLOTS + slot) as u64);
        let mut p = self.params.patterns.get(v, slot);
        f(&mut p);
        self.params.patterns.set(v, slot, &p);
    }

    /// A change to the pattern voice `v` edits that is an undo step of its own.
    fn edit_action(&self, v: usize, f: impl FnOnce(&mut Pattern)) {
        self.checkpoint_action();
        self.restoring.set(true);
        self.edit(v, f);
        self.restoring.set(false);
    }

    fn unit(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures, v: usize) {
        let (x, y) = (34.0, 398.0 + v as f32 * 152.0);
        let col = rgb(VOICE_COLORS[v]);
        let outer = pen.r(x, y, 1212.0, 148.0);
        metal_plate(pen, t, outer, rgb(0xb9bcc1));
        // the lower, dark part, as on the 303
        let low = pen.r(x + 1.0, y + 84.0, 1210.0, 63.0);
        pen.vgradient(low, rgb(0x2a2b2f), rgb(0x1a1b1e));
        pen.line(
            pen.p(x + 1.0, y + 84.0),
            pen.p(x + 1211.0, y + 84.0),
            1.0,
            rgba(0x000000, 200),
        );
        pen.line(
            pen.p(x + 1.0, y + 85.0),
            pen.p(x + 1211.0, y + 85.0),
            1.0,
            rgba(0xffffff, 30),
        );
        for (sx, sy) in [(x + 8.0, y + 8.0), (x + 1204.0, y + 8.0)] {
            screw(pen, pen.p(sx, sy));
        }

        // name plate, model and waveform
        let plate = pen.r(x + 16.0, y + 8.0, 112.0, 28.0);
        well(pen, plate, rgb(0x161718), 3.0);
        pen.text(
            x + 26.0,
            y + 22.0,
            Align2::LEFT_CENTER,
            "TB",
            BOLD,
            10.0,
            rgb(0x8c8e93),
        );
        pen.text(
            x + 44.0,
            y + 22.0,
            Align2::LEFT_CENTER,
            "303",
            BOLD,
            17.0,
            rgb(0xe8e8e8),
        );
        pen.text(
            x + 118.0,
            y + 22.0,
            Align2::RIGHT_CENTER,
            &format!("{}", v + 1),
            BOLD,
            20.0,
            col,
        );
        let on = self.value(voice(v, V::On)) >= 0.5;
        let level = self.meters[v].min(1.0).sqrt();
        led(
            pen,
            pen.p(x + 98.0, y + 22.0),
            3.0,
            col,
            if on { 0.25 + 0.75 * level } else { 0.0 },
        );
        self.selector(
            ui,
            pen,
            pen.r(x + 16.0, y + 42.0, 112.0, 18.0),
            voice(v, V::Model),
            9.0,
            rgb(0xffc04a),
            None,
        );
        let square = self.value(voice(v, V::Wave)) >= 0.5;
        for (k, (name, lit)) in [("SAW", !square), ("SQUARE", square)]
            .into_iter()
            .enumerate()
        {
            let br = pen.r(x + 16.0 + k as f32 * 58.0, y + 64.0, 54.0, 15.0);
            let (clicked, down) = self.click(ui, br, ("wave", v, k));
            if clicked {
                self.set(voice(v, V::Wave), k as f32);
            }
            button(pen, br, name, 7.5, rgb(0xd8dade), lit.then_some(col), down);
        }

        // the sound knobs
        let knobs = [
            (V::Tune, "TUNE"),
            (V::Fine, "FINE"),
            (V::Cutoff, "CUT OFF FREQ"),
            (V::Resonance, "RESONANCE"),
            (V::EnvMod, "ENV MOD"),
            (V::Decay, "DECAY"),
            (V::Accent, "ACCENT"),
        ];
        for (k, (c, name)) in knobs.into_iter().enumerate() {
            let kx = x + 182.0 + k as f32 * 70.0;
            let r = if k < 2 { 17.0 } else { 20.0 };
            self.big_knob(ui, pen, kx, y + 50.0, r, voice(v, c), name, col);
        }
        let tune = self.value(voice(v, V::Tune)) as i32;
        let fine = self.value(voice(v, V::Fine));
        lcd(
            pen,
            pen.r(x + 164.0, y + 71.0, 36.0, 12.0),
            &format!("{tune:+} st"),
            7.5,
            rgb(0x7dffa0),
        );
        lcd(
            pen,
            pen.r(x + 232.0, y + 71.0, 40.0, 12.0),
            &format!("{fine:+.0} ct"),
            7.5,
            rgb(0x7dffa0),
        );
        pen.line(
            pen.p(x + 655.0, y + 8.0),
            pen.p(x + 655.0, y + 78.0),
            1.0,
            rgba(0x000000, 70),
        );
        pen.line(
            pen.p(x + 656.0, y + 8.0),
            pen.p(x + 656.0, y + 78.0),
            1.0,
            rgba(0xffffff, 120),
        );

        // the generator
        pen.engraved(
            x + 668.0,
            y + 12.0,
            Align2::LEFT_CENTER,
            "GENERATOR",
            8.5,
            rgb(0x222326),
            rgba(0xffffff, 150),
        );
        let gens = [
            (V::Density, "DENSITY"),
            (V::Accents, "ACCENTS"),
            (V::Slides, "SLIDES"),
            (V::Octaves, "OCTAVES"),
            (V::Evolve, "EVOLVE"),
        ];
        for (k, (c, name)) in gens.into_iter().enumerate() {
            let kx = x + 688.0 + k as f32 * 48.0;
            let i = voice(v, c);
            let rect = Rect::from_center_size(pen.p(kx, y + 52.0), vec2(34.0, 34.0) * pen.scale);
            let active = self.drag_control(ui, pen, rect, i, 200.0);
            let tval = self.value(i);
            pen.engraved(
                kx,
                y + 30.0,
                Align2::CENTER_CENTER,
                name,
                7.0,
                rgb(0x222326),
                rgba(0xffffff, 150),
            );
            knob_scale(pen, pen.p(kx, y + 52.0), 12.0, 7, rgba(0x2a2b2e, 200));
            knob(pen, pen.p(kx, y + 52.0), 12.0, tval, col, active);
        }
        let gen_rect = pen.r(x + 930.0, y + 22.0, 70.0, 24.0);
        let (generate, down) = self.click(ui, gen_rect, ("gen", v));
        button(pen, gen_rect, "GENERATE", 8.5, rgb(0xd03a22), None, down);
        let mut_rect = pen.r(x + 930.0, y + 52.0, 70.0, 24.0);
        let (mutate, mdown) = self.click(ui, mut_rect, ("mut", v));
        button(pen, mut_rect, "MUTATE", 8.5, rgb(0x4a4c52), None, mdown);
        let (root, scale) = self.scale();
        let g = self.gen_params(v);
        if generate {
            let mut rng = self.rng;
            self.edit_action(v, |p| generator::generate(p, &g, root, scale, &mut rng));
            self.rng.next_u64();
            self.rng = Rng::new(rng.next_u64());
        }
        if mutate {
            let mut rng = Rng::new(self.rng.next_u64());
            self.edit_action(v, |p| generator::mutate(p, &g, 0.25, root, scale, &mut rng));
        }

        // the patterns
        pen.engraved(
            x + 1016.0,
            y + 12.0,
            Align2::LEFT_CENTER,
            "PATTERN",
            8.5,
            rgb(0x222326),
            rgba(0xffffff, 150),
        );
        let slot = self.slot(v);
        let playing = self.feedback.active[v].load(Relaxed) as usize;
        let running = self.feedback.playing.load(Relaxed);
        for s in 0..SLOTS {
            let br = pen.r(
                x + 1016.0 + (s % 4) as f32 * 46.0,
                y + 24.0 + (s / 4) as f32 * 28.0,
                42.0,
                22.0,
            );
            let (clicked, down) = self.click(ui, br, ("slot", v, s));
            if clicked {
                self.set(voice(v, V::Pattern), (s + 1) as f32);
            }
            button(
                pen,
                br,
                &format!("{}", s + 1),
                10.0,
                rgb(0xd8dade),
                (s == slot).then_some(col),
                down,
            );
            if running && s == playing {
                led(
                    pen,
                    br.right_top() + vec2(-pen.s(6.0), pen.s(6.0)),
                    2.6,
                    rgb(0xff3020),
                    1.0,
                );
            }
        }

        // the step editor
        let pattern = self.params.patterns.get(v, slot);
        let len = pattern.len as usize;
        let current = if running && playing == slot {
            self.feedback.steps[v].load(Relaxed)
        } else {
            -1
        };
        pen.text(
            x + 16.0,
            y + 94.0,
            Align2::LEFT_CENTER,
            "LENGTH",
            BOLD,
            7.0,
            rgb(0x8c8e93),
        );
        let minus = pen.r(x + 16.0, y + 100.0, 24.0, 18.0);
        let (less, ld) = self.click(ui, minus, ("len-", v));
        button(pen, minus, "−", 11.0, rgb(0x4a4c52), None, ld);
        lcd(
            pen,
            pen.r(x + 44.0, y + 100.0, 40.0, 18.0),
            &format!("{len}"),
            10.5,
            rgb(0xffa81c),
        );
        let plus = pen.r(x + 88.0, y + 100.0, 24.0, 18.0);
        let (more, md) = self.click(ui, plus, ("len+", v));
        button(pen, plus, "+", 11.0, rgb(0x4a4c52), None, md);
        // the step rate, and whether MIDI notes transpose this voice
        self.selector(
            ui,
            pen,
            pen.r(x + 16.0, y + 124.0, 62.0, 18.0),
            voice(v, V::Rate),
            8.5,
            rgb(0xffa81c),
            None,
        );
        let follow = self.value(voice(v, V::MidiTranspose)) >= 0.5;
        let br = pen.r(x + 82.0, y + 124.0, 30.0, 18.0);
        let p = self.toggle(ui, br, voice(v, V::MidiTranspose));
        button(
            pen,
            br,
            "MIDI",
            7.0,
            rgb(0x3a3c40),
            follow.then_some(rgb(0x7dffa0)),
            p,
        );
        if less && len > 1 {
            self.edit_action(v, |p| p.len -= 1);
        }
        if more && len < MAX_STEPS {
            self.edit_action(v, |p| p.len += 1);
        }
        let evolve = self.value(voice(v, V::Evolve));
        pen.text(
            x + 760.0,
            y + 12.0,
            Align2::LEFT_CENTER,
            if evolve > 0.0 { "EVOLVING" } else { "" },
            BOLD,
            7.5,
            col,
        );

        let (root, scale) = self.scale();
        for i in 0..MAX_STEPS {
            let cx = x + 132.0 + i as f32 * 59.0;
            let step = pattern.steps[i];
            let inside = i < len;
            if i % 4 == 0 {
                pen.fill(
                    pen.r(cx - 1.0, y + 89.0, 4.0 * 59.0 - 3.0, 55.0),
                    3.0,
                    rgba(0xffffff, if i % 8 == 0 { 10 } else { 5 }),
                );
            }
            led(
                pen,
                pen.p(cx + 28.0, y + 94.0),
                3.0,
                rgb(0xff3020),
                if current == i as i32 { 1.0 } else { 0.0 },
            );
            pen.text(
                cx + 7.0,
                y + 94.0,
                Align2::LEFT_CENTER,
                &format!("{}", i + 1),
                BOLD,
                6.5,
                rgb(0x6c6e73),
            );
            // the note: click to play or rest, drag or turn to change
            let nr = pen.r(cx + 2.0, y + 101.0, 52.0, 22.0);
            let resp = ui.interact(nr, key(("note", v, i)), Sense::click_and_drag());
            let shown = if scale == Scale::Chromatic {
                step.note
            } else {
                quantize(step.note, root, scale)
            };
            if resp.clicked() {
                self.edit_action(v, |p| p.steps[i].rest = !p.steps[i].rest);
            }
            if resp.drag_started() {
                // a drag is one undo step, apart from what came before
                self.history.borrow_mut().seal();
                self.note_drag = Some(NoteDrag {
                    voice: v,
                    step: i,
                    start: shown,
                    travel: 0.0,
                });
            }
            if resp.dragged()
                && let Some(d) = &mut self.note_drag
                && d.voice == v
                && d.step == i
            {
                d.travel += -resp.drag_delta().y / pen.scale;
                let by = (d.travel / 7.0) as i32;
                let start = d.start;
                let note = self.move_in_scale(start, by);
                self.edit(v, |p| {
                    p.steps[i].note = note;
                    p.steps[i].rest = false;
                });
            }
            if resp.drag_stopped() {
                self.note_drag = None;
            }
            if resp.hovered() && !resp.dragged() {
                let scroll = ui.input(|inp| inp.smooth_scroll_delta.y);
                if scroll.abs() > 0.5 {
                    let note = self.move_in_scale(shown, scroll.signum() as i32);
                    self.edit(v, |p| p.steps[i].note = note);
                }
            }
            let name = note_name(shown);
            let ink = if step.rest {
                rgba(0xffa81c, 60)
            } else {
                rgb(0xffa81c)
            };
            lcd(pen, nr, if step.rest { "· · ·" } else { &name }, 11.0, ink);
            if step.rest {
                pen.painter.text(
                    nr.center() + vec2(0.0, pen.s(7.5)),
                    Align2::CENTER_CENTER,
                    &name,
                    pen.font(BOLD, 6.0),
                    rgba(0xffa81c, 70),
                );
            }
            for (k, (label, on, lit)) in [
                ("AC", step.accent, rgb(0xff4b2b)),
                ("SL", step.slide, rgb(0x52d2ff)),
            ]
            .into_iter()
            .enumerate()
            {
                let br = pen.r(cx + 2.0 + k as f32 * 27.0, y + 127.0, 25.0, 15.0);
                let (clicked, down) = self.click(ui, br, ("flag", v, i, k));
                if clicked {
                    self.edit_action(v, |p| {
                        if k == 0 {
                            p.steps[i].accent = !p.steps[i].accent;
                        } else {
                            p.steps[i].slide = !p.steps[i].slide;
                        }
                    });
                }
                button(pen, br, label, 7.0, rgb(0x55575d), on.then_some(lit), down);
            }
            if !inside {
                pen.fill(pen.r(cx, y + 88.0, 57.0, 57.0), 3.0, rgba(0x101012, 190));
            }
        }

        // pattern tools, and copying patterns between voices and slots
        let tools = [
            ("←", 0),
            ("→", 1),
            ("CLR", 2),
            ("COPY", 6),
            ("−1", 3),
            ("+1", 4),
            ("OCT", 5),
            ("PASTE", 7),
        ];
        for (n, (label, k)) in tools.into_iter().enumerate() {
            let br = pen.r(
                x + 1081.0 + (n % 4) as f32 * 31.5,
                y + 101.0 + (n / 4) as f32 * 24.0,
                29.0,
                19.0,
            );
            let (clicked, down) = self.click(ui, br, ("tool", v, k));
            let copied_here = matches!(self.clipboard, Some((_, cv, cs)) if cv == v && cs == slot);
            let lit = match k {
                6 if copied_here => Some(col),
                7 if self.clipboard.is_some() => Some(rgb(0x6a6c72)),
                _ => None,
            };
            let size = if label.len() > 3 { 6.5 } else { 8.5 };
            button(pen, br, label, size, rgb(0x4a4c52), lit, down);
            if !clicked {
                continue;
            }
            match k {
                6 => self.clipboard = Some((self.params.patterns.get(v, slot), v, slot)),
                7 => {
                    if let Some((copied, _, _)) = self.clipboard {
                        self.edit_action(v, |p| *p = copied);
                    }
                }
                _ => self.edit_action(v, |p| match k {
                    0 => generator::rotate(p, -1),
                    1 => generator::rotate(p, 1),
                    2 => {
                        for s in &mut p.steps {
                            *s = front303_core::Step::default();
                        }
                    }
                    3 => generator::transpose(p, -1),
                    4 => generator::transpose(p, 1),
                    _ => generator::transpose(p, 12),
                }),
            }
        }
    }

    /// The bottom bar: the ten memories, UNDO and REDO.
    fn memory_bar(&mut self, ui: &mut Ui, pen: &Pen, t: &Textures) {
        let y = 1012.0;
        let r = pen.r(34.0, y, 1212.0, 40.0);
        dark_plate(pen, t, r, rgb(0x2b2c30), rgb(0x18191b));
        pen.text(
            48.0,
            y + 20.0,
            Align2::LEFT_CENTER,
            "MEMORY",
            BOLD,
            9.0,
            rgb(0x8c8e93),
        );
        let now = Instant::now();
        for m in 0..MEMORIES {
            let br = pen.r(112.0 + m as f32 * 66.0, y + 6.0, 60.0, 28.0);
            let resp = ui.interact(br, key(("memory", m)), Sense::click_and_drag());
            // a click quicker than a frame is only seen as a click: a recall
            let quick = resp.clicked() && self.hold.is_none();
            if quick {
                self.recall_memory(m);
            }
            if resp.is_pointer_button_down_on() && self.hold.is_none() && !quick {
                self.hold = Some(Hold {
                    slot: m,
                    since: now,
                    key: None,
                    saved: false,
                });
            }
            let used = self.memory_used(m);
            let current = used && self.current_memory == Some(m);
            let flash = self.saved_at[m]
                .map(|at| (1.0 - now.duration_since(at).as_secs_f32() / 0.8).max(0.0))
                .unwrap_or(0.0);
            let face = if current {
                rgb(ORANGE)
            } else if used {
                rgb(0x55402c)
            } else {
                rgb(0x2a2b2e)
            };
            pen.fill(br, 3.0, face);
            if resp.hovered() {
                pen.fill(br, 3.0, rgba(0xffffff, 18));
            }
            pen.stroke(br, 3.0, 1.0, rgba(0x60626a, 255));
            let ink = if current {
                rgb(0x141414)
            } else if used {
                rgb(0xf0f0f0)
            } else {
                rgb(0x6c6e73)
            };
            let number = if m == 9 {
                "0".to_string()
            } else {
                format!("{}", m + 1)
            };
            let (bx, by, bw, _) = pen.design(br);
            pen.text(
                bx + bw / 2.0,
                by + 9.0,
                Align2::CENTER_CENTER,
                &number,
                BOLD,
                10.0,
                ink,
            );
            let note = if flash > 0.0 {
                "SAVED"
            } else if used {
                "recall"
            } else {
                "empty"
            };
            pen.text(
                bx + bw / 2.0,
                by + 21.0,
                Align2::CENTER_CENTER,
                note,
                MEDIUM,
                7.0,
                ink,
            );
            if flash > 0.0 {
                pen.fill(br, 3.0, rgba(0xffffff, (150.0 * flash) as u8));
            }
            if let Some(h) = &self.hold
                && h.slot == m
                && !h.saved
            {
                let progress = (now.duration_since(h.since).as_secs_f32()
                    / HOLD_TO_SAVE.as_secs_f32())
                .min(1.0);
                pen.fill(
                    pen.r(bx, by + 24.0, bw * progress, 4.0),
                    1.0,
                    rgba(0xffffff, 110),
                );
            }
        }
        // the press in progress: released early it recalls, held it saves
        if let Some(h) = &self.hold {
            let down = match h.key {
                Some(k) => ui.input(|i| i.key_down(k)),
                None => ui.input(|i| i.pointer.primary_down()),
            };
            let (slot, held, saved) = (h.slot, now.duration_since(h.since), h.saved);
            if down {
                if !saved && held >= HOLD_TO_SAVE {
                    self.save_memory(slot);
                    if let Some(h) = &mut self.hold {
                        h.saved = true;
                    }
                }
            } else {
                if !saved {
                    self.recall_memory(slot);
                }
                self.hold = None;
            }
        }

        // undo and redo
        let (can_undo, can_redo) = {
            let h = self.history.borrow();
            (h.can_undo(), h.can_redo())
        };
        let br = pen.r(790.0, y + 8.0, 64.0, 24.0);
        let (undo, down) = self.click(ui, br, "undo");
        button(
            pen,
            br,
            "UNDO",
            8.5,
            rgb(0x4a4c52),
            can_undo.then_some(rgb(0x8a8c92)),
            down,
        );
        let br = pen.r(860.0, y + 8.0, 64.0, 24.0);
        let (redo, down) = self.click(ui, br, "redo");
        button(
            pen,
            br,
            "REDO",
            8.5,
            rgb(0x4a4c52),
            can_redo.then_some(rgb(0x8a8c92)),
            down,
        );
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        let clip = match self.clipboard {
            Some((_, v, s)) => format!("COPIED: 303-{} · PATTERN {}", v + 1, s + 1),
            None => "COPY / PASTE: PATTERN TOOLS".to_string(),
        };
        let help = [
            "CLICK OR KEY 1-0: RECALL  ·  HOLD 0.6 s: SAVE",
            "CTRL+Z UNDO  ·  CTRL+SHIFT+Z / CTRL+Y REDO",
        ];
        pen.text(
            938.0,
            y + 11.0,
            Align2::LEFT_CENTER,
            help[0],
            MEDIUM,
            7.0,
            rgb(0x8c8e93),
        );
        pen.text(
            938.0,
            y + 21.0,
            Align2::LEFT_CENTER,
            help[1],
            MEDIUM,
            7.0,
            rgb(0x8c8e93),
        );
        pen.text(
            938.0,
            y + 31.0,
            Align2::LEFT_CENTER,
            &clip,
            BOLD,
            7.0,
            rgb(0xffa81c),
        );
    }

    /// The keyboard: digits for the memories, Ctrl+Z and Ctrl+Shift+Z or Ctrl+Y.
    fn keys(&mut self, ui: &mut Ui) {
        use egui::{Key, Modifiers};
        let redo = ui.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || i.consume_key(Modifiers::COMMAND, Key::Y)
        });
        let undo = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z));
        if redo {
            self.redo();
        } else if undo {
            self.undo();
        }
        if self.hold.is_none() {
            let pressed = ui.input(|i| {
                if i.modifiers.any() {
                    return None;
                }
                DIGITS.iter().position(|&k| i.key_pressed(k))
            });
            if let Some(m) = pressed {
                self.hold = Some(Hold {
                    slot: m,
                    since: Instant::now(),
                    key: Some(DIGITS[m]),
                    saved: false,
                });
            }
        }
    }

    fn draw(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let Some(t) = self.textures.take() else {
            return;
        };
        let full = ctx.content_rect();
        let scale = (full.width() / W).min(full.height() / H);
        let origin = full.center() - vec2(W, H) * scale / 2.0;
        let pen = Pen {
            painter: ui.painter().clone(),
            scale,
            origin,
        };
        pen.painter.rect_filled(full, 0.0, rgb(0x0c0c0d));
        // the rack behind the units
        pen.vgradient(pen.r(26.0, 0.0, 1228.0, H), rgb(0x1d1e21), rgb(0x111214));
        cheek(&pen, &t, pen.r(0.0, 0.0, 26.0, H), false);
        cheek(&pen, &t, pen.r(W - 26.0, 0.0, 26.0, H), true);

        // meters fall slowly, like the original's
        for k in 0..VOICES + 3 {
            let peak = self.feedback.take_peak(k);
            self.meters[k] = peak.max(self.meters[k] * 0.86);
        }
        self.bubble = None;
        self.header(ui, &pen, &t);
        self.effects(ui, &pen, &t);
        self.reverb(ui, &pen, &t);
        self.mixer(ui, &pen, &t);
        for v in 0..VOICES {
            self.unit(ui, &pen, &t, v);
        }
        self.keys(ui);
        self.memory_bar(ui, &pen, &t);
        if let Some((at, text)) = self.bubble.take() {
            let font = pen.font(BOLD, 9.0);
            let galley = pen.painter.layout_no_wrap(text, font, rgb(0xf2f2f2));
            let size = galley.size() + vec2(12.0, 6.0) * scale;
            let r = Rect::from_center_size(at - vec2(0.0, size.y / 2.0 + 4.0 * scale), size);
            pen.fill(
                r.translate(vec2(0.0, 2.0 * scale)),
                3.0,
                rgba(0x000000, 120),
            );
            pen.fill(r, 3.0, rgba(0x18191b, 235));
            pen.stroke(r, 3.0, 1.0, rgba(ORANGE, 200));
            pen.painter
                .galley(r.center() - galley.size() / 2.0, galley, Color32::WHITE);
        }
        self.textures = Some(t);
        ctx.request_repaint_after(Duration::from_millis(30));
    }
}

/// A widget's identity: the same every frame, whatever the panel's layout.
fn key(salt: impl std::hash::Hash + std::fmt::Debug) -> Id {
    Id::NULL.with(salt)
}

fn note_name(note: i8) -> String {
    let n = note as i32;
    format!(
        "{}{}",
        controls::NOTES[n.rem_euclid(12) as usize],
        2 + n.div_euclid(12)
    )
}

fn fonts(ctx: &Context) {
    let mut f = FontDefinitions::default();
    f.font_data.insert(
        BOLD.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../fonts/Inter-Bold.ttf"
        ))),
    );
    f.font_data.insert(
        MEDIUM.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../fonts/Inter-Medium.ttf"
        ))),
    );
    let fallback: Vec<String> = f
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for name in [BOLD, MEDIUM] {
        let mut list = vec![name.to_string()];
        list.extend(fallback.iter().cloned());
        f.families.insert(FontFamily::Name(name.into()), list);
    }
    ctx.set_fonts(f);
}

impl NiceEguiApp for Panel {
    fn build(
        &mut self,
        ctx: Context,
        gui: GuiContext,
        _frame: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        fonts(&ctx);
        self.textures = Some(Textures::new(&ctx));
        self.gui = Some(gui);
        Ok(())
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut nice_plug_egui::Frame) {
        if self.textures.is_none() {
            self.textures = Some(Textures::new(ui.ctx()));
        }
        self.draw(ui);
    }

    fn editor_closed(&mut self) {
        self.textures = None;
        self.gesture = None;
        self.note_drag = None;
        self.hold = None;
    }
}
