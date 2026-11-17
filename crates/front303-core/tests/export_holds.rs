//! Writes held C1 notes for comparison with other emulations:
//! `FRONT303_HOLDS=DIR cargo test --release --test export_holds -- --ignored`.
mod common;
use common::*;
use front303_core::*;
use std::io::Write;

#[test]
#[ignore]
fn export_holds() {
    let dir = std::env::var("FRONT303_HOLDS").unwrap_or_else(|_| ".".into());
    for (model, name) in [(Model::Dw303, "dw303"), (Model::Rebirth, "rebirth")] {
        for cut in [0.0f32, 0.5, 1.0] {
            for reso in [0.0f32, 1.0] {
                let mut e = held(model, -12);
                let v = &mut e.params.voices[0];
                (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (cut, reso, 0.0, 0.0, 0.0);
                e.play();
                let (l, _) = render(&mut e, 1.2);
                let path = format!("{dir}/{name}_hold_c{cut:.1}_r{reso:.0}.f32");
                let mut f = std::fs::File::create(path).unwrap();
                for x in l {
                    f.write_all(&x.to_le_bytes()).unwrap();
                }
            }
        }
    }
}

fn write(path: String, x: &[f32]) {
    let mut f = std::fs::File::create(path).unwrap();
    for v in x {
        f.write_all(&v.to_le_bytes()).unwrap();
    }
}

#[test]
#[ignore]
fn export_dynamics() {
    let dir = std::env::var("FRONT303_HOLDS").unwrap_or_else(|_| ".".into());
    for (model, name) in [(Model::Dw303, "dw303"), (Model::Rebirth, "rebirth")] {
        // eight accented sixteenths then rests, at 120 BPM
        for reso in [0.0f32, 1.0] {
            let mut e = held(model, 0);
            let mut steps = vec![Step::note(0).accent(); 8];
            steps.extend(vec![Step::default(); 8]);
            e.patterns[0][0] = Pattern::from_steps(&steps);
            e.params.bpm = 120.0;
            let v = &mut e.params.voices[0];
            (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (0.3, reso, 0.5, 0.3, 1.0);
            e.play();
            let (l, _) = render(&mut e, 1.0);
            write(format!("{dir}/{name}_accent_r{reso:.0}.f32"), &l);
        }
        // C2 sliding into C3 at 60 BPM, filter open
        let mut e = held(model, 0);
        e.patterns[0][0] = Pattern::from_steps(&[Step::note(0).slide(), Step::note(12).slide()]);
        e.params.bpm = 60.0;
        let v = &mut e.params.voices[0];
        (v.cutoff, v.resonance, v.env_mod) = (1.0, 0.0, 0.0);
        e.play();
        let (l, _) = render(&mut e, 0.6);
        write(format!("{dir}/{name}_slide.f32"), &l);
        // a plain note then an accented one, accent full, same everything else
        let mut e = held(model, 0);
        e.patterns[0][0] = Pattern::from_steps(&[
            Step::note(0),
            Step::default(),
            Step::default(),
            Step::default(),
            Step::note(0).accent(),
            Step::default(),
            Step::default(),
            Step::default(),
        ]);
        e.params.bpm = 120.0;
        let v = &mut e.params.voices[0];
        (v.cutoff, v.resonance, v.env_mod, v.decay, v.accent) = (0.4, 0.5, 0.5, 0.5, 1.0);
        e.play();
        let (l, _) = render(&mut e, 1.0);
        write(format!("{dir}/{name}_plain_accent.f32"), &l);
    }
}
