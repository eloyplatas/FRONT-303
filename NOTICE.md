# Notice

Front 303 is free software under the GNU General Public License, version 3 only.

It is built on two projects, used as libraries at pinned revisions (`Cargo.toml`):

- **OpenRB-338** (https://github.com/pingw33n/OpenRB-338), GPL-3.0-only, copyright the
  OpenRB-338 authors: the `orb-engine` crate. Front 303 uses its TB-303 device, its tables, its
  distortion, PCF, compressor and delay unchanged, and follows its documentation of ReBirth's
  mixer block in `crates/front303-core/src/engine.rs`. OpenRB-338 is a reimplementation of
  Propellerhead's ReBirth RB-338 2.01, made by reverse engineering.
- **DW-303** (https://github.com/mo0kid/DW-303), GPL-3.0-or-later, copyright DJW Audio: the
  `dw303-osc`, `dw303-vcf`, `dw303-env` and `dw303-voice` crates, and `ravet-signal` from RAVET
  (https://github.com/mo0kid/ravet). `crates/front303-core/src/dw.rs` composes the circuit
  stages the way `dw303-voice` does, with the tuning added.

Since 0.3.0 Front 303 also contains, in `third_party/` (compiled in by `crates/front303-cxx`),
or ported to Rust:

- **Open303** (https://github.com/maddanio/open303, a fork of Robin Schmidt's Open303), MIT,
  copyright Robin Schmidt and contributors: `third_party/open303`, used as the Open303 voice.
  Changed: two out-of-range accesses in `rosic_MipMappedWaveTable` corrected (marked
  "Front 303").
- **freeverb3** (Teru Kamogashira), GPL-2.0-or-later, as distributed with **Dragonfly Reverb**
  (https://github.com/michaelwillis/dragonfly-reverb, Michael Willis and Rob van den Berg,
  GPL-3.0-or-later): `third_party/freeverb3`. `crates/front303-cxx/shim/dragonfly.cpp` follows
  Dragonfly Hall's and Dragonfly Early Reflections' DSP code and uses programs from their
  banks. Changed: buffer allocation goes through a recycling pool (`fv3_rt_pool`), and one
  buffer the original never released is released.
- **Aether** (https://github.com/Dougal-s/Aether, Dougal Stewart), MIT: `third_party/aether`,
  without its LV2 wrapper; the LV2 headers it includes (`third_party/lv2`, ISC) come with it.
  The eight programs are Front 303's.
- **CloudReverb** (https://github.com/xunil-cloud/CloudReverb), GPL-3.0, with the CloudSeed
  engine (Valdemar Erlingsson, Ghost Note Audio; MIT): `third_party/cloudseed`, the audio
  engine and the presets.
  Changed: the seed generator writes into fixed buffers instead of allocating.
- **IV-X Delay** (https://github.com/qbroquetas/IV-XDelay, Joaquim Broquetas Ruiz), GPL-3.0:
  ported to Rust in `crates/front303-core/src/fx/tape.rs`; its shaper and FIR tables are in
  `crates/front303-core/assets/ivx_*.bin`, converted from its source.
- **Setekh** (https://github.com/fullfxmedia/setekh, Full FX Media), GPL-3.0: ported to Rust in
  `fx/setekh.rs`.
- **Valentine** (https://github.com/tote-bag-labs/valentine, Tote Bag Labs), GPL-3.0: ported to
  Rust in `fx/valentine.rs`.
- **Delay Intervals** (https://github.com/Matth-ewe-f/audio-plugin-delay-intervals, Matthew
  Fecher), MIT: `fx/intervals.rs` follows its processing; the patterns are Front 303's.

Each project's licence file is kept beside its sources in `third_party/`, and in the release
archive under `licenses/`.

The plugin is built with nice-plug (https://codeberg.org/RustAudio/nice-plug, ISC) and egui
(https://github.com/emilk/egui, MIT or Apache-2.0). Its VST3 support uses the `vst3` bindings,
generated from Steinberg's VST3 SDK (MIT). The panel's typeface is Inter (copyright The Inter
Project Authors, SIL Open Font License 1.1, `crates/front303-plugin/fonts/OFL.txt`).

The panel's artwork is Front 303's own, drawn in code; it copies no picture of ReBirth or of
the TB-303.

Front 303 uses none of these projects' artwork or samples. It contains nothing of ReBirth
itself.

ReBirth RB-338 is a trademark of Propellerhead Software (now Reason Studios). TB-303 and Roland
are trademarks of Roland Corporation. DW-303 and DJW Audio are names of their owner. Front 303
is an independent project, not affiliated with or endorsed by any of them.
