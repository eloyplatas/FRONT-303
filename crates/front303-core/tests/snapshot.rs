//! Undo history and memories.
use std::time::{Duration, Instant};

use front303_core::controls;
use front303_core::snapshot::{History, MEMORIES, Memories, PATTERNS, Snapshot};
use front303_core::{Pattern, Step};

fn snapshot(seed: f32) -> Snapshot {
    let mut values = controls::defaults();
    values[controls::voice(0, controls::V::Cutoff)] = seed;
    let mut patterns = vec![Pattern::default(); PATTERNS];
    patterns[3] = Pattern::from_steps(&[Step::note(5).accent(), Step::note(-7).slide()]);
    Snapshot { values, patterns }
}

#[test]
fn a_snapshot_survives_its_text() {
    let s = snapshot(0.25);
    assert_eq!(Snapshot::from_text(&s.to_text()), Some(s));
}

#[test]
fn memories_load_by_identifier() {
    let mut m = Memories::default();
    m.slots[2] = Some(snapshot(0.75));
    m.slots[9] = Some(snapshot(0.1));
    let text = m.to_text();
    assert_eq!(Memories::from_text(&text), m);
    // a control this version does not know is skipped; one it lacks keeps its default
    let older = text
        .replace("v1_cutoff=0.75;", "v1_cutoff=0.75;gone=3;")
        .replace("v2_reverb=0;", "");
    let back = Memories::from_text(&older);
    assert_eq!(back.slots[2], m.slots[2]);
    assert!(back.slots.iter().filter(|s| s.is_some()).count() == 2);
    assert_eq!(MEMORIES, 10);
}

#[test]
fn undo_and_redo_walk_the_history() {
    let mut h = History::default();
    let t0 = Instant::now();
    // a drag: three changes under one tag share one checkpoint
    h.checkpoint(1, t0, || snapshot(0.0));
    h.checkpoint(1, t0 + Duration::from_millis(200), || snapshot(9.0));
    h.checkpoint(1, t0 + Duration::from_millis(400), || snapshot(9.0));
    // another control
    h.checkpoint(2, t0 + Duration::from_millis(500), || snapshot(0.5));
    let back = h.undo(snapshot(0.7)).unwrap();
    assert_eq!(back, snapshot(0.5));
    let back = h.undo(back).unwrap();
    assert_eq!(back, snapshot(0.0));
    assert!(!h.can_undo());
    let forward = h.redo(back).unwrap();
    assert_eq!(forward, snapshot(0.5));
    let forward = h.redo(forward).unwrap();
    assert_eq!(forward, snapshot(0.7));
    assert!(!h.can_redo());
    // a new change after an undo drops the redo stack
    h.undo(forward);
    h.checkpoint(3, t0 + Duration::from_secs(5), || snapshot(0.2));
    assert!(!h.can_redo());
    // the same tag after a pause starts a new step
    h.checkpoint(3, t0 + Duration::from_secs(9), || snapshot(0.3));
    assert_eq!(h.undo(snapshot(1.0)).unwrap(), snapshot(0.3));
}
