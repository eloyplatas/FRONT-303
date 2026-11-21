//! Front 303: four TB-303 voices, each running the ReBirth RB-338 engine (OpenRB-338's
//! reimplementation), the DW-303 circuit model or Open303, through ReBirth's mixer,
//! distortion, PCF, compressor and delay, with Setekh, Valentine, tape and interval modes on
//! the effects and a reverb with four engines.
//!
//! This crate is the sound engine only and knows nothing about plugins or windows:
//! [`Engine`] holds [`Params`] and four [`Pattern`]s and renders stereo audio at
//! [`SAMPLE_RATE`].

#![allow(clippy::needless_range_loop, clippy::field_reassign_with_default)]
#![cfg_attr(not(feature = "engine"), allow(dead_code))]

pub mod controls;

#[cfg(feature = "engine")]
mod dw;
#[cfg(feature = "engine")]
mod engine;
#[cfg(feature = "engine")]
mod fx;
pub mod generator;
#[cfg(feature = "engine")]
mod o3;
pub mod params;
pub mod pattern;
#[cfg(feature = "engine")]
mod rb;
pub mod shared;
pub mod snapshot;

#[cfg(feature = "engine")]
pub use engine::{Engine, FX_OUTPUT, OUTPUTS, SAMPLE_RATE, TRANSPOSE_RANGE, voice_output};
#[cfg(feature = "engine")]
pub use fx::intervals::pattern_name as interval_pattern_name;
#[cfg(feature = "engine")]
pub use fx::reverb::program_name as reverb_program_name;
pub use generator::Rng;
pub use params::*;
pub use pattern::{MAX_STEPS, NOTE_MAX, NOTE_MIN, Pattern, Step};
pub use shared::{Feedback, PatternBank};
