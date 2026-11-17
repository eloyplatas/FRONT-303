#![allow(clippy::field_reassign_with_default)]
//! The control table and the shared pattern bank.
use front303_core::controls::{self, G, Kind, V};
use front303_core::*;

#[test]
fn the_table_is_complete_and_its_ids_are_unique() {
    let c = controls::controls();
    assert_eq!(c.len(), controls::COUNT);
    let mut ids: Vec<&str> = c.iter().map(|x| x.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), controls::COUNT);
    for x in &c {
        assert!(x.min() < x.max(), "{}", x.id);
        assert_eq!(x.snap(x.default), x.default, "{}", x.id);
        assert!(!x.text(x.default).is_empty());
    }
    assert_eq!(c[controls::voice(2, V::Cutoff)].id, "v3_cutoff");
    assert_eq!(c[controls::global(G::DelayPan)].id, "delay_pan");
    assert_eq!(
        c[controls::voice(0, V::Tune)].kind,
        Kind::Int { min: -24, max: 24 }
    );
}

/// The ids hosts store automation under. A change here breaks saved projects.
#[test]
fn the_ids_do_not_change() {
    let ids: Vec<String> = controls::controls().into_iter().map(|c| c.id).collect();
    let voice = "model wave tune fine cutoff reso envmod decay accent pattern on level pan send \
                 dist density accents slides octaves evolve rate miditr reverb";
    let mut expected = Vec::new();
    for v in 1..=4 {
        expected.extend(voice.split_whitespace().map(|x| format!("v{v}_{x}")));
    }
    let globals = "run tempo shuffle root scale master quality dist_amount dist_shape \
                   pcf_channel pcf_freq pcf_reso pcf_amount pcf_pattern pcf_decay pcf_mode \
                   comp_route comp_amount comp_threshold delay_on delay_steps delay_triplet \
                   delay_feedback delay_pan dist_mode dist_level comp_mode comp_ratio \
                   comp_attack comp_release comp_saturate comp_crush comp_mix delay_mode \
                   delay_a delay_b delay_c delay_kind rev_on rev_mode rev_program rev_predelay \
                   rev_size rev_decay rev_damping rev_mod rev_width rev_delay rev_level output";
    expected.extend(globals.split_whitespace().map(String::from));
    assert_eq!(ids, expected);
}

#[test]
fn the_defaults_give_the_default_settings() {
    let mut p = Params::default();
    controls::apply(&controls::defaults(), &mut p);
    let mut expected = Params::default();
    // the table's default scale is minor and only its first channel is on; everything else is
    // the engine's default
    expected.scale = Scale::Minor;
    for c in 1..VOICES {
        expected.channels[c].on = false;
    }
    assert_eq!(p, expected);
}

#[test]
fn values_reach_the_settings() {
    let mut v = controls::defaults();
    v[controls::voice(1, V::Model)] = 2.0;
    v[controls::voice(1, V::Tune)] = -24.0;
    v[controls::voice(3, V::Pattern)] = 8.0;
    v[controls::global(G::PcfChannel)] = 4.0;
    v[controls::global(G::CompRoute)] = 3.0;
    v[controls::global(G::Root)] = 9.0;
    v[controls::global(G::PcfPattern)] = 55.0;
    let mut p = Params::default();
    controls::apply(&v, &mut p);
    assert_eq!(p.voices[1].model, Model::Dw303);
    assert_eq!(p.voices[1].tune, -24);
    assert_eq!(p.voices[3].pattern, 7);
    assert_eq!(p.pcf.channel, Some(3));
    assert_eq!(p.comp.route, CompRoute::Channel(1));
    assert_eq!(p.root, 9);
    assert_eq!(p.pcf.pattern, 54);
}

#[test]
fn the_bank_round_trips_through_its_text() {
    let bank = PatternBank::default();
    let mut rng = Rng::new(5);
    for v in 0..VOICES {
        for s in 0..SLOTS {
            let mut p = Pattern::default();
            p.len = 1 + ((v * 5 + s * 3) % 16) as u8;
            generator::generate(&mut p, &GenParams::default(), 4, Scale::Blues, &mut rng);
            bank.set(v, s, &p);
        }
    }
    let text = bank.serialize();
    let other = PatternBank::default();
    let before = other.edits();
    assert!(other.deserialize(&text));
    assert_ne!(other.edits(), before);
    for v in 0..VOICES {
        for s in 0..SLOTS {
            assert_eq!(other.get(v, s), bank.get(v, s));
        }
    }
    assert!(!other.deserialize("nonsense"));
    assert!(!other.deserialize(&text[1..]));
    assert_eq!(other.serialize(), text);
}
