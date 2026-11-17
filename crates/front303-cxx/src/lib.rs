//! Front 303's C++ engines behind safe handles: Open303's voice, and the Dragonfly Hall,
//! Dragonfly Early Reflections, Aether and CloudSeed reverbs.
//!
//! Every handle owns its engine and is used from one thread at a time.

#![allow(unsafe_code)]

use core::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    fn f303_o3_new(sample_rate: f64) -> *mut c_void;
    fn f303_o3_free(v: *mut c_void);
    fn f303_o3_panel(
        v: *mut c_void,
        cutoff_hz: f64,
        resonance: f64,
        env_mod: f64,
        decay_ms: f64,
        accent: f64,
        waveform: f64,
        tuning_a4_hz: f64,
    );
    fn f303_o3_trigger(v: *mut c_void, note: c_int, accent: c_int);
    fn f303_o3_slide(v: *mut c_void, note: c_int, accent: c_int);
    fn f303_o3_release(v: *mut c_void);
    fn f303_o3_silence(v: *mut c_void);
    fn f303_o3_render(v: *mut c_void, out: *mut f32, n: c_int);

    fn f303_rev_new(kind: c_int, rate: f64) -> *mut c_void;
    fn f303_rev_free(e: *mut c_void);
    fn f303_rev_program(e: *mut c_void, p: c_int);
    fn f303_rev_shape(e: *mut c_void, size: f64, decay: f64, damping: f64, modulation: f64);
    fn f303_rev_clear(e: *mut c_void);
    fn f303_rev_process(
        e: *mut c_void,
        il: *const f32,
        ir: *const f32,
        ol: *mut f32,
        or: *mut f32,
        n: c_int,
    );
    fn f303_rev_program_name(kind: c_int, p: c_int) -> *const c_char;
}

/// Open303's voice, played note by note.
pub struct Open303 {
    ptr: *mut c_void,
}

// SAFETY: the engine has no thread affinity; the handle is used from one thread at a time.
unsafe impl Send for Open303 {}

/// The panel of an Open303 voice, in the units of Open303's own plug-in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Open303Panel {
    pub cutoff_hz: f64,
    pub resonance: f64,
    pub env_mod: f64,
    pub decay_ms: f64,
    pub accent: f64,
    /// 0 saw, 1 square.
    pub waveform: f64,
    /// The frequency of A4.
    pub tuning_hz: f64,
}

impl Open303 {
    pub fn new(sample_rate: f64) -> Self {
        // SAFETY: plain construction; the pointer is owned by the handle.
        let ptr = unsafe { f303_o3_new(sample_rate) };
        assert!(!ptr.is_null());
        Open303 { ptr }
    }

    pub fn panel(&mut self, p: &Open303Panel) {
        // SAFETY: `ptr` is a live engine owned by this handle.
        unsafe {
            f303_o3_panel(
                self.ptr,
                p.cutoff_hz,
                p.resonance,
                p.env_mod,
                p.decay_ms,
                p.accent,
                p.waveform,
                p.tuning_hz,
            )
        }
    }

    /// Starts a note (MIDI number).
    pub fn trigger(&mut self, note: i32, accent: bool) {
        // SAFETY: as above.
        unsafe { f303_o3_trigger(self.ptr, note, accent as c_int) }
    }

    /// Glides the held note to another.
    pub fn slide(&mut self, note: i32, accent: bool) {
        // SAFETY: as above.
        unsafe { f303_o3_slide(self.ptr, note, accent as c_int) }
    }

    pub fn release(&mut self) {
        // SAFETY: as above.
        unsafe { f303_o3_release(self.ptr) }
    }

    pub fn silence(&mut self) {
        // SAFETY: as above.
        unsafe { f303_o3_silence(self.ptr) }
    }

    pub fn render(&mut self, out: &mut [f32]) {
        // SAFETY: as above; `out` is valid for its length.
        unsafe { f303_o3_render(self.ptr, out.as_mut_ptr(), out.len() as c_int) }
    }
}

impl Drop for Open303 {
    fn drop(&mut self) {
        // SAFETY: the engine is freed once, here.
        unsafe { f303_o3_free(self.ptr) }
    }
}

/// The reverb engines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReverbKind {
    /// Dragonfly Hall: freeverb3's early reflections into its zrev2 late reverb.
    Hall,
    /// Dragonfly Early Reflections.
    Early,
    /// Aether.
    Aether,
    /// CloudReverb (CloudSeed).
    Cloud,
}

impl ReverbKind {
    pub const ALL: [ReverbKind; 4] = [
        ReverbKind::Hall,
        ReverbKind::Early,
        ReverbKind::Aether,
        ReverbKind::Cloud,
    ];

    /// The name of one of the eight programs.
    pub fn program_name(self, program: usize) -> &'static str {
        // SAFETY: the shim returns pointers to static, NUL-terminated ASCII strings.
        unsafe {
            let p = f303_rev_program_name(self as c_int, program as c_int);
            core::ffi::CStr::from_ptr(p).to_str().unwrap_or("?")
        }
    }
}

/// Programs per engine.
pub const REVERB_PROGRAMS: usize = 8;
/// The most samples one call to [`Reverb::process`] takes.
pub const REVERB_BLOCK: usize = 64;

/// One reverb engine, wet output only.
pub struct Reverb {
    ptr: *mut c_void,
}

// SAFETY: as for Open303.
unsafe impl Send for Reverb {}

impl Reverb {
    /// Builds the engine; this allocates and is not for the audio thread.
    pub fn new(kind: ReverbKind, sample_rate: f64) -> Self {
        // SAFETY: plain construction; the pointer is owned by the handle.
        let ptr = unsafe { f303_rev_new(kind as c_int, sample_rate) };
        assert!(!ptr.is_null());
        Reverb { ptr }
    }

    pub fn program(&mut self, program: usize) {
        // SAFETY: `ptr` is a live engine owned by this handle.
        unsafe { f303_rev_program(self.ptr, program as c_int) }
    }

    /// The character knobs, 0..1, 0.5 leaving the program as it is.
    pub fn shape(&mut self, size: f32, decay: f32, damping: f32, modulation: f32) {
        // SAFETY: as above.
        unsafe {
            f303_rev_shape(
                self.ptr,
                size as f64,
                decay as f64,
                damping as f64,
                modulation as f64,
            )
        }
    }

    pub fn clear(&mut self) {
        // SAFETY: as above.
        unsafe { f303_rev_clear(self.ptr) }
    }

    /// Up to [`REVERB_BLOCK`] samples.
    pub fn process(&mut self, il: &[f32], ir: &[f32], ol: &mut [f32], or: &mut [f32]) {
        let n = il
            .len()
            .min(ir.len())
            .min(ol.len())
            .min(or.len())
            .min(REVERB_BLOCK);
        // SAFETY: as above; every slice holds at least `n` samples.
        unsafe {
            f303_rev_process(
                self.ptr,
                il.as_ptr(),
                ir.as_ptr(),
                ol.as_mut_ptr(),
                or.as_mut_ptr(),
                n as c_int,
            )
        }
    }
}

impl Drop for Reverb {
    fn drop(&mut self) {
        // SAFETY: the engine is freed once, here.
        unsafe { f303_rev_free(self.ptr) }
    }
}
