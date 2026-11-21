//! Whole-instrument snapshots: the panel's undo history and the ten memories.
//!
//! A snapshot holds every control's value and every pattern of every voice. Memories keep the
//! values under the controls' identifiers, so a memory saved by one version still loads into a
//! later one that has more controls.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::controls::{self, Control, G};
use crate::params::{SLOTS, VOICES};
use crate::pattern::{MAX_STEPS, NOTE_MAX, NOTE_MIN, Pattern, Step};

/// Patterns in a snapshot: every slot of every voice, voice by voice.
pub const PATTERNS: usize = VOICES * SLOTS;
/// Memories.
pub const MEMORIES: usize = 10;
/// Undo steps kept.
pub const UNDO_DEPTH: usize = 100;

pub(crate) fn pack(s: Step) -> u16 {
    let note = (s.note.clamp(NOTE_MIN, NOTE_MAX) - NOTE_MIN) as u16;
    note | (s.accent as u16) << 6 | (s.slide as u16) << 7 | (s.rest as u16) << 8
}

pub(crate) fn unpack(x: u16) -> Step {
    Step {
        note: ((x & 0x3F) as i8 + NOTE_MIN).clamp(NOTE_MIN, NOTE_MAX),
        accent: x & 0x40 != 0,
        slide: x & 0x80 != 0,
        rest: x & 0x100 != 0,
    }
}

const HEX_PER_PATTERN: usize = 2 + MAX_STEPS * 3;

/// Patterns as text: per pattern its length and sixteen steps, in hexadecimal.
pub fn patterns_to_hex(patterns: &[Pattern]) -> String {
    let mut out = String::with_capacity(patterns.len() * HEX_PER_PATTERN);
    for p in patterns {
        out.push_str(&format!("{:02x}", p.len.clamp(1, MAX_STEPS as u8)));
        for step in p.steps {
            out.push_str(&format!("{:03x}", pack(step)));
        }
    }
    out
}

/// Reads [`patterns_to_hex`]'s text; `None` unless it holds exactly `count` patterns.
pub fn patterns_from_hex(text: &str, count: usize) -> Option<Vec<Pattern>> {
    if text.len() != count * HEX_PER_PATTERN || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = Vec::with_capacity(count);
    for k in 0..count {
        let chunk = &text[k * HEX_PER_PATTERN..][..HEX_PER_PATTERN];
        let mut p = Pattern::default();
        p.len = u8::from_str_radix(&chunk[..2], 16)
            .unwrap_or(16)
            .clamp(1, MAX_STEPS as u8);
        for (i, step) in p.steps.iter_mut().enumerate() {
            let x = u16::from_str_radix(&chunk[2 + i * 3..][..3], 16).unwrap_or(0x100);
            *step = unpack(x);
        }
        out.push(p);
    }
    Some(out)
}

/// Whether a snapshot sets control `i`. The transport's RUN switch, the output mode and the
/// DW-303 solver stay as they are: they belong to the session, not to the music.
pub fn restores(i: usize) -> bool {
    i != controls::global(G::Run)
        && i != controls::global(G::Output)
        && i != controls::global(G::Quality)
}

/// Every control's value and every pattern.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub values: Vec<f32>,
    pub patterns: Vec<Pattern>,
}

impl Snapshot {
    /// The pattern of voice `v` in slot `s`.
    pub fn pattern(&self, v: usize, s: usize) -> &Pattern {
        &self.patterns[v * SLOTS + s]
    }

    /// The snapshot as text, under Front 303's control identifiers.
    pub fn to_text(&self) -> String {
        self.to_text_with(&controls::controls())
    }

    /// The snapshot as text: `id=value` pairs for `table`'s controls, then the patterns in
    /// hexadecimal.
    pub fn to_text_with(&self, table: &[Control]) -> String {
        let mut out = String::new();
        for (c, v) in table.iter().zip(&self.values) {
            out.push_str(&format!("{}={};", c.id, v));
        }
        out.push('|');
        out.push_str(&patterns_to_hex(&self.patterns));
        out
    }

    /// Reads [`Snapshot::to_text`]'s text.
    pub fn from_text(text: &str) -> Option<Snapshot> {
        Self::from_text_with(text, &controls::controls())
    }

    /// Reads [`Snapshot::to_text_with`]'s text for `table`. Controls it does not name take
    /// their defaults; identifiers `table` does not have are ignored.
    pub fn from_text_with(text: &str, table: &[Control]) -> Option<Snapshot> {
        let (values_text, hex) = text.split_once('|')?;
        let mut values: Vec<f32> = table.iter().map(|c| c.default).collect();
        for pair in values_text.split(';').filter(|p| !p.is_empty()) {
            let (id, v) = pair.split_once('=')?;
            let v: f32 = v.parse().ok()?;
            if let Some(i) = table.iter().position(|c| c.id == id) {
                values[i] = table[i].snap(v);
            }
        }
        let patterns = patterns_from_hex(hex, PATTERNS)?;
        Some(Snapshot { values, patterns })
    }
}

/// The ten memories.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memories {
    pub slots: [Option<Snapshot>; MEMORIES],
}

impl Memories {
    /// One line per used memory, under Front 303's control identifiers.
    pub fn to_text(&self) -> String {
        self.to_text_with(&controls::controls())
    }

    /// One line per used memory: its number, a colon and the snapshot's text.
    pub fn to_text_with(&self, table: &[Control]) -> String {
        let mut out = String::new();
        for (i, s) in self.slots.iter().enumerate() {
            if let Some(s) = s {
                out.push_str(&format!("{i}:{}\n", s.to_text_with(table)));
            }
        }
        out
    }

    pub fn from_text(text: &str) -> Memories {
        Self::from_text_with(text, &controls::controls())
    }

    /// Reads [`Memories::to_text_with`]'s text; lines it cannot read stay empty memories.
    pub fn from_text_with(text: &str, table: &[Control]) -> Memories {
        let mut m = Memories::default();
        for line in text.lines() {
            if let Some((i, rest)) = line.split_once(':')
                && let Ok(i) = i.parse::<usize>()
                && i < MEMORIES
            {
                m.slots[i] = Snapshot::from_text_with(rest, table);
            }
        }
        m
    }
}

/// The panel's undo and redo stacks.
///
/// A checkpoint is the state before a change. Changes that follow one another closely under
/// the same tag (one knob's drag, one step's note being dragged, quick wheel turns) share one
/// checkpoint, so one undo takes back the whole gesture.
pub struct History {
    undo: VecDeque<Snapshot>,
    redo: Vec<Snapshot>,
    last: Option<(u64, Instant)>,
}

/// How long changes under one tag keep sharing a checkpoint.
const COALESCE: Duration = Duration::from_millis(600);

impl Default for History {
    fn default() -> Self {
        History {
            undo: VecDeque::with_capacity(UNDO_DEPTH),
            redo: Vec::new(),
            last: None,
        }
    }
}

impl History {
    /// Notes that a change tagged `tag` is about to be made to the state `before` (called
    /// lazily: `before` runs only when a new checkpoint is needed).
    pub fn checkpoint(&mut self, tag: u64, now: Instant, before: impl FnOnce() -> Snapshot) {
        let joins = matches!(self.last, Some((t, at)) if t == tag && now - at < COALESCE);
        self.last = Some((tag, now));
        if joins {
            return;
        }
        if self.undo.len() == UNDO_DEPTH {
            self.undo.pop_front();
        }
        self.undo.push_back(before());
        self.redo.clear();
    }

    /// Ends any gesture in progress: the next change takes its own checkpoint.
    pub fn seal(&mut self) {
        self.last = None;
    }

    /// The state to go back to, given the present one.
    pub fn undo(&mut self, present: Snapshot) -> Option<Snapshot> {
        let back = self.undo.pop_back()?;
        self.redo.push(present);
        self.last = None;
        Some(back)
    }

    /// The state to go forward to, given the present one.
    pub fn redo(&mut self, present: Snapshot) -> Option<Snapshot> {
        let forward = self.redo.pop()?;
        self.undo.push_back(present);
        self.last = None;
        Some(forward)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
