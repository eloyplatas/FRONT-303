//! Patterns and the events a voice plays.

/// The most steps a pattern holds.
pub const MAX_STEPS: usize = 16;
/// The lowest and highest step note, in semitones from C2 (65.4 Hz): the 303's keyboard
/// octave with its octave-down and octave-up switches.
pub const NOTE_MIN: i8 = -12;
pub const NOTE_MAX: i8 = 24;

/// One sixteenth step, with ReBirth's meaning: a slide ties this step into the next one
/// (the gate stays open and the next pitch glides).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    /// Semitones from C2.
    pub note: i8,
    pub accent: bool,
    pub slide: bool,
    pub rest: bool,
}

impl Default for Step {
    fn default() -> Self {
        Step {
            note: 0,
            accent: false,
            slide: false,
            rest: true,
        }
    }
}

impl Step {
    pub fn note(note: i8) -> Self {
        Step {
            note,
            accent: false,
            slide: false,
            rest: false,
        }
    }

    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }

    pub fn slide(mut self) -> Self {
        self.slide = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub steps: [Step; MAX_STEPS],
    /// 1..=16.
    pub len: u8,
}

impl Default for Pattern {
    /// Sixteen rests.
    fn default() -> Self {
        Pattern {
            steps: [Step::default(); MAX_STEPS],
            len: MAX_STEPS as u8,
        }
    }
}

impl Pattern {
    pub fn from_steps(steps: &[Step]) -> Self {
        let mut p = Pattern::default();
        let n = steps.len().clamp(1, MAX_STEPS);
        p.steps[..steps.len().min(MAX_STEPS)].copy_from_slice(&steps[..steps.len().min(MAX_STEPS)]);
        p.len = n as u8;
        p
    }

    pub(crate) fn step(&self, index: i64) -> Step {
        let len = (self.len as usize).clamp(1, MAX_STEPS) as i64;
        self.steps[index.rem_euclid(len) as usize]
    }
}

/// What a voice is told, at a sample of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    /// A step begins.
    Step(Step),
    /// Half a step later: the gate closes unless the step slides.
    GateOff,
    /// The transport stopped: the gate closes whatever the step.
    AllOff,
}

/// The events of one voice in one block, in order, each with its sample offset.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Events {
    items: [Option<(u8, Event)>; 6],
    len: usize,
}

impl Events {
    pub(crate) fn push(&mut self, offset: usize, event: Event) {
        if self.len < self.items.len() {
            self.items[self.len] = Some((offset as u8, event));
            self.len += 1;
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, Event)> + '_ {
        self.items[..self.len]
            .iter()
            .flatten()
            .map(|&(o, e)| (o as usize, e))
    }
}
