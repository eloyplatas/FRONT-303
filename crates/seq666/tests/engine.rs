//! The MIDI sequencer: timing, gates, slides, channels and transposition.

use std::collections::HashMap;

use front303_core::params::{Rate, Scale};
use front303_core::{Pattern, Step};
use seq666::controls::CHANNELS;
use seq666::engine::{NoteEvent, Sequencer};

const RATE: f64 = 48_000.0;
/// A sixteenth at 120 BPM and 48 kHz.
const SIXTEENTH: u64 = 6_000;

/// A sequencer at 120 BPM with only channel 0 on, playing `p`, chromatic.
fn seq_with(p: Pattern) -> Sequencer {
    let mut s = Sequencer::new();
    s.params.bpm = 120.0;
    s.params.scale = Scale::Chromatic;
    for c in 0..CHANNELS {
        s.params.channels[c].on = c == 0;
    }
    s.patterns[0][0] = p;
    s
}

/// Runs `samples` in blocks of `block`, returning every event with its absolute sample.
fn run(s: &mut Sequencer, samples: u64, block: usize) -> Vec<(u64, NoteEvent)> {
    let mut out = Vec::new();
    let mut at = 0u64;
    while at < samples {
        let n = block.min((samples - at) as usize);
        s.process(n, RATE, |e| out.push((at + e.timing as u64, e)));
        at += n as u64;
    }
    out
}

#[test]
fn notes_start_on_the_step_and_end_after_the_gate() {
    let mut s = seq_with(Pattern::from_steps(&[Step::note(0); 16]));
    s.params.channels[0].gate = 0.5;
    s.play();
    let ev = run(&mut s, SIXTEENTH * 4, 512);
    let times: Vec<(u64, bool)> = ev.iter().map(|(t, e)| (*t, e.on)).collect();
    let expect: Vec<(u64, bool)> = (0..4)
        .flat_map(|k| {
            [
                (k * SIXTEENTH, true),
                (k * SIXTEENTH + SIXTEENTH / 2, false),
            ]
        })
        .collect();
    assert_eq!(times.len(), expect.len());
    for ((t, on), (et, eon)) in times.iter().zip(&expect) {
        assert_eq!(on, eon);
        assert!(t.abs_diff(*et) <= 1, "{t} vs {et}");
    }
    // the pattern's C2 is MIDI note 36, on the channel's MIDI channel
    assert!(ev.iter().all(|(_, e)| e.note == 36 && e.channel == 0));
}

#[test]
fn channel_octave_velocity_and_accent() {
    let mut s = seq_with(Pattern::from_steps(&[
        Step::note(3),
        Step::note(3).accent(),
    ]));
    let c = &mut s.params.channels[0];
    c.midi_channel = 9;
    c.octave = 1;
    c.velocity = 80;
    c.accent_velocity = 120;
    s.play();
    let ons: Vec<NoteEvent> = run(&mut s, SIXTEENTH * 2, 256)
        .into_iter()
        .map(|(_, e)| e)
        .filter(|e| e.on)
        .collect();
    assert_eq!(ons.len(), 2);
    assert!(ons.iter().all(|e| e.channel == 9 && e.note == 36 + 12 + 3));
    assert_eq!((ons[0].velocity, ons[1].velocity), (80, 120));
}

#[test]
fn a_slide_overlaps_the_next_note() {
    let mut s = seq_with(Pattern::from_steps(&[
        Step::note(0).slide(),
        Step::note(5),
        Step::default(),
        Step::default(),
    ]));
    s.play();
    let ev = run(&mut s, SIXTEENTH * 4, 1000);
    let seq: Vec<(bool, u8)> = ev.iter().map(|(_, e)| (e.on, e.note)).collect();
    // on 36, then on 41 before off 36, then off 41 at the second step's gate
    assert_eq!(seq, vec![(true, 36), (true, 41), (false, 36), (false, 41)]);
    assert!(ev[1].0.abs_diff(SIXTEENTH) <= 1);
    assert_eq!(ev[1].0, ev[2].0);
}

#[test]
fn a_slide_to_the_same_note_holds_it() {
    let mut s = seq_with(Pattern::from_steps(&[
        Step::note(0).slide(),
        Step::note(0),
        Step::default(),
        Step::default(),
    ]));
    s.play();
    let ev = run(&mut s, SIXTEENTH * 4, 333);
    assert_eq!(ev.len(), 2);
    assert!(ev[0].1.on && !ev[1].1.on);
    assert!(ev[1].0 > SIXTEENTH);
}

#[test]
fn midi_transpose_moves_only_the_channels_that_follow() {
    let mut s = seq_with(Pattern::from_steps(&[Step::note(0)]));
    s.params.channels[1].on = true;
    s.params.channels[1].midi_transpose = false;
    s.patterns[1][0] = Pattern::from_steps(&[Step::note(0)]);
    s.transpose = 7;
    s.play();
    let ons: Vec<NoteEvent> = run(&mut s, SIXTEENTH, 64)
        .into_iter()
        .map(|(_, e)| e)
        .filter(|e| e.on)
        .collect();
    assert!(ons.iter().any(|e| e.channel == 0 && e.note == 43));
    assert!(ons.iter().any(|e| e.channel == 1 && e.note == 36));
}

#[test]
fn rates_and_shuffle() {
    let mut s = seq_with(Pattern::from_steps(&[Step::note(0); 16]));
    s.params.channels[0].rate = Rate::ALL[3];
    let ratio = Rate::ALL[3].sixteenths();
    s.play();
    let ons: Vec<u64> = run(&mut s, SIXTEENTH * 8, 500)
        .into_iter()
        .filter(|(_, e)| e.on)
        .map(|(t, _)| t)
        .collect();
    let step = (SIXTEENTH as f64 * ratio).round() as u64;
    assert!(ons.windows(2).all(|w| (w[1] - w[0]).abs_diff(step) <= 1));

    let mut s = seq_with(Pattern::from_steps(&[Step::note(0); 16]));
    s.params.shuffle = 1.0;
    s.play();
    let ons: Vec<u64> = run(&mut s, SIXTEENTH * 4, 500)
        .into_iter()
        .filter(|(_, e)| e.on)
        .map(|(t, _)| t)
        .collect();
    // every second step starts half a step late
    assert_eq!(ons.len(), 4);
    assert!(ons[1].abs_diff(SIXTEENTH + SIXTEENTH / 2) <= 1);
    assert!(ons[2].abs_diff(2 * SIXTEENTH) <= 1);
}

#[test]
fn stopping_switching_off_and_changing_channel_release_every_note() {
    let mut s = seq_with(Pattern::from_steps(&[Step::note(0).slide(); 16]));
    s.params.channels[0].midi_channel = 3;
    s.play();
    let mut ev = run(&mut s, 1000, 100);
    // the MIDI channel changes while the note sounds: the note ends where it started
    s.params.channels[0].midi_channel = 5;
    s.params.channels[0].on = false;
    ev.extend(run(&mut s, 100, 100));
    assert_eq!(ev.len(), 2);
    assert_eq!((ev[1].1.on, ev[1].1.channel), (false, 3));

    s.params.channels[0].on = true;
    ev = run(&mut s, SIXTEENTH * 2, 100);
    assert!(ev.iter().any(|(_, e)| e.on && e.channel == 5));
    s.stop();
    ev.extend(run(&mut s, 100, 100));
    assert!(!ev.last().unwrap().1.on);
    assert!(run(&mut s, SIXTEENTH * 4, 100).is_empty());
}

#[test]
fn every_note_on_gets_its_note_off() {
    let mut s = Sequencer::new();
    s.params.bpm = 173.0;
    s.params.shuffle = 0.4;
    let mut rng = front303_core::Rng::new(7);
    for c in 0..CHANNELS {
        s.params.channels[c].gate = [0.05, 0.5, 0.9, 1.0][c];
        s.params.channels[c].rate = Rate::ALL[c % Rate::ALL.len()];
        s.params.channels[c].generator.evolve = 0.5;
        for p in 0..8 {
            let mut g = front303_core::GenParams::default();
            g.slides = 0.5;
            front303_core::generator::generate(
                &mut s.patterns[c][p],
                &g,
                0,
                Scale::Minor,
                &mut rng,
            );
        }
    }
    s.play();
    let mut held: HashMap<(u8, u8), i32> = HashMap::new();
    let mut count = 0;
    for k in 0..400 {
        if k % 50 == 25 {
            s.transpose = (k % 7) - 3;
            s.params.channels[k as usize % CHANNELS].pattern = (k % 8) as u8;
        }
        if k == 200 {
            s.locate(37.5);
        }
        s.process(441, 44_100.0, |e| {
            let n = held.entry((e.channel, e.note)).or_default();
            *n += if e.on { 1 } else { -1 };
            // never two of the same note at once, never an off without its on
            assert!((0..=1).contains(n), "{e:?}");
            count += 1;
        });
    }
    s.stop();
    s.process(64, 44_100.0, |e| {
        *held.entry((e.channel, e.note)).or_default() += if e.on { 1 } else { -1 };
    });
    assert!(count > 100);
    assert!(held.values().all(|&n| n == 0));
}
