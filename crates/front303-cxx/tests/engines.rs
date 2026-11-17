//! The C++ engines: they run, they sound, and once built they do not touch the heap.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use front303_cxx::{Open303, Open303Panel, REVERB_BLOCK, REVERB_PROGRAMS, Reverb, ReverbKind};

// Every malloc (Rust's and C++'s) of the counting thread is counted while COUNTING is set;
// other tests run on other threads at the same time.
static COUNTING: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static THREAD: AtomicUsize = AtomicUsize::new(0);

fn counted() -> bool {
    COUNTING.load(Ordering::Relaxed) && unsafe { pthread_self() } == THREAD.load(Ordering::Relaxed)
}

unsafe extern "C" {
    fn pthread_self() -> usize;
    fn __libc_malloc(size: usize) -> *mut u8;
    fn __libc_calloc(n: usize, size: usize) -> *mut u8;
    fn __libc_realloc(p: *mut u8, size: usize) -> *mut u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    if counted() {
        CALLS.fetch_add(1, Ordering::Relaxed);
    }
    unsafe { __libc_malloc(size) }
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(n: usize, size: usize) -> *mut u8 {
    if counted() {
        CALLS.fetch_add(1, Ordering::Relaxed);
    }
    unsafe { __libc_calloc(n, size) }
}

#[unsafe(no_mangle)]
/// # Safety
/// As C's `realloc`.
pub unsafe extern "C" fn realloc(p: *mut u8, size: usize) -> *mut u8 {
    if counted() {
        CALLS.fetch_add(1, Ordering::Relaxed);
    }
    unsafe { __libc_realloc(p, size) }
}

const RATE: f64 = 44100.0;

fn panel() -> Open303Panel {
    Open303Panel {
        cutoff_hz: 800.0,
        resonance: 70.0,
        env_mod: 50.0,
        decay_ms: 500.0,
        accent: 50.0,
        waveform: 0.0,
        tuning_hz: 440.0,
    }
}

#[test]
fn open303_plays_a_note() {
    let mut v = Open303::new(RATE);
    v.panel(&panel());
    v.trigger(48, false);
    let mut out = vec![0.0f32; 8820];
    v.render(&mut out);
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(peak > 0.05 && peak.is_finite(), "peak {peak}");
}

fn impulse_energy(r: &mut Reverb) -> (f32, f32) {
    let mut il = [0.0f32; REVERB_BLOCK];
    let mut ir = [0.0f32; REVERB_BLOCK];
    let mut ol = [0.0f32; REVERB_BLOCK];
    let mut or = [0.0f32; REVERB_BLOCK];
    let mut energy = 0.0f32;
    let mut late = 0.0f32;
    for block in 0..(RATE as usize * 3 / REVERB_BLOCK) {
        il.fill(0.0);
        ir.fill(0.0);
        if block == 0 {
            il[0] = 1.0;
            ir[0] = 1.0;
        }
        r.process(&il, &ir, &mut ol, &mut or);
        for i in 0..REVERB_BLOCK {
            let e = ol[i] * ol[i] + or[i] * or[i];
            assert!(e.is_finite());
            energy += e;
            if block * REVERB_BLOCK > RATE as usize {
                late += e;
            }
        }
    }
    (energy, late)
}

#[test]
fn every_reverb_program_rings() {
    for kind in ReverbKind::ALL {
        let mut r = Reverb::new(kind, RATE);
        for p in 0..REVERB_PROGRAMS {
            r.program(p);
            r.clear();
            let (energy, late) = impulse_energy(&mut r);
            println!(
                "{kind:?} {:<26} energy {:6.2} dB, after 1 s {:6.2} dB",
                kind.program_name(p),
                10.0 * energy.log10(),
                10.0 * late.max(1e-12).log10()
            );
            assert!(energy > 1e-4, "{kind:?} {p} is silent");
        }
    }
}

#[test]
fn engines_do_not_allocate_once_built() {
    let mut reverbs: Vec<Reverb> = ReverbKind::ALL
        .iter()
        .map(|&k| Reverb::new(k, RATE))
        .collect();
    let mut voice = Open303::new(RATE);
    let il = [0.1f32; REVERB_BLOCK];
    let mut ol = [0.0f32; REVERB_BLOCK];
    let mut or = [0.0f32; REVERB_BLOCK];
    let mut out = [0.0f32; REVERB_BLOCK];
    // every program and the knob extremes once before counting: the pool keeps that memory
    for r in &mut reverbs {
        for p in 0..REVERB_PROGRAMS {
            r.program(p);
            for k in [0.0, 1.0, 0.5] {
                r.shape(k, k, k, k);
                r.process(&il, &il, &mut ol, &mut or);
            }
        }
    }
    THREAD.store(unsafe { pthread_self() }, Ordering::SeqCst);
    COUNTING.store(true, Ordering::SeqCst);
    for step in 0..400usize {
        let k = (step % 37) as f32 / 36.0;
        for r in &mut reverbs {
            r.program(step / 50 % REVERB_PROGRAMS);
            r.shape(k, 1.0 - k, k * 0.5, k);
            r.process(&il, &il, &mut ol, &mut or);
        }
        voice.panel(&panel());
        if step % 8 == 0 {
            voice.trigger(36 + (step % 24) as i32, step % 3 == 0);
        } else if step % 8 == 3 {
            voice.slide(40, false);
        } else if step % 8 == 5 {
            voice.release();
        }
        voice.render(&mut out);
    }
    COUNTING.store(false, Ordering::SeqCst);
    assert_eq!(
        CALLS.load(Ordering::SeqCst),
        0,
        "heap allocations on the audio path"
    );
}
