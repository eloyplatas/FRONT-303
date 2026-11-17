//! Rendering must not allocate: it runs on a host's audio thread.
mod common;
use common::*;
use front303_core::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};

struct Counting;
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static SIZES: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if COUNTING.load(SeqCst) {
            ALLOCATIONS.fetch_add(1, SeqCst);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

// The C++ engines allocate through malloc: count that too.
unsafe extern "C" {
    fn __libc_malloc(size: usize) -> *mut u8;
    fn __libc_calloc(n: usize, size: usize) -> *mut u8;
    fn __libc_realloc(p: *mut u8, size: usize) -> *mut u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    if COUNTING.load(SeqCst) {
        let i = ALLOCATIONS.fetch_add(1, SeqCst);
        if i < 8 {
            SIZES[i].store(size, SeqCst);
        }
    }
    unsafe { __libc_malloc(size) }
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(n: usize, size: usize) -> *mut u8 {
    if COUNTING.load(SeqCst) {
        ALLOCATIONS.fetch_add(1, SeqCst);
    }
    unsafe { __libc_calloc(n, size) }
}

#[unsafe(no_mangle)]
/// # Safety
/// As C's `realloc`.
pub unsafe extern "C" fn realloc(p: *mut u8, size: usize) -> *mut u8 {
    if COUNTING.load(SeqCst) {
        ALLOCATIONS.fetch_add(1, SeqCst);
    }
    unsafe { __libc_realloc(p, size) }
}

#[test]
fn rendering_does_not_allocate() {
    let mut e = Engine::new();
    for v in 0..VOICES {
        e.patterns[v][0] = riff();
        e.patterns[v][1] = riff();
        e.params.voices[v].model = MODELS[v % 4];
        e.params.voices[v].rate = Rate::ALL[v * 2];
        e.params.channels[v].reverb = 0.5;
        e.params.voices[v].generator.evolve = 0.5;
        e.params.channels[v].dist = v % 2 == 0;
        e.params.channels[v].send = 60;
    }
    e.params.pcf.channel = Some(1);
    e.params.comp.route = CompRoute::Master;
    e.params.scale = Scale::Dorian;
    e.params.shuffle = 0.4;
    e.params.dw_quality = DwQuality::Economy;
    let (mut l, mut r) = (vec![0.0f32; 4096], vec![0.0f32; 4096]);
    let mut stems = vec![vec![0.0f32; 4096]; OUTPUTS - 2];
    e.play();
    // a self-check: the counter sees an allocation
    COUNTING.store(true, SeqCst);
    drop(std::hint::black_box(vec![0u8; 100]));
    COUNTING.store(false, SeqCst);
    assert!(
        ALLOCATIONS.swap(0, SeqCst) > 0,
        "the counter is not counting"
    );
    COUNTING.store(true, SeqCst);
    for n in 0..120 {
        if n == 20 {
            for v in 0..VOICES {
                e.params.voices[v].pattern = 1;
                e.params.voices[v].model = MODELS[(v + 1) % 4];
                e.params.voices[v].rate = Rate::ALL[8 - v];
            }
            e.transpose = 5;
        }
        if n == 40 {
            e.sync(true, 100.0);
        }
        // every effect mode and every reverb program in turn
        e.params.dist.mode = [DistMode::Rebirth, DistMode::Setekh][n / 10 % 2];
        e.params.comp.mode = [CompMode::Rebirth, CompMode::Valentine][n / 7 % 2];
        e.params.comp.route = [CompRoute::Master, CompRoute::Channel(2)][n / 13 % 2];
        e.params.comp.valentine.crush = (n % 3) as u8;
        e.params.delay.mode =
            [DelayMode::Rebirth, DelayMode::Tape, DelayMode::Intervals][n / 5 % 3];
        e.params.delay.steps = 1 + (n % 7) as u8;
        e.params.delay.kind = (n % 12) as u8;
        e.params.reverb.mode = ReverbMode::ALL[n / 4 % 4];
        e.params.reverb.program = (n / 2 % 8) as u8;
        e.params.reverb.size = (n % 11) as f32 / 10.0;
        e.params.reverb.decay = (n % 5) as f32 / 4.0;
        e.params.output = [Output::Stereo, Output::Multi][n / 30 % 2];
        if n % 2 == 0 {
            e.render(&mut l, &mut r);
        } else {
            COUNTING.store(false, SeqCst);
            let (a, rest) = stems.split_at_mut(1);
            let mut outs: Vec<&mut [f32]> = Vec::new();
            outs.push(&mut l);
            outs.push(&mut r);
            outs.push(&mut a[0]);
            for s in rest.iter_mut() {
                outs.push(s);
            }
            // building the slice list allocates; only the render is counted
            let mut outs = std::hint::black_box(outs);
            COUNTING.store(true, SeqCst);
            e.render_outputs(&mut outs);
            COUNTING.store(false, SeqCst);
            drop(outs);
            COUNTING.store(true, SeqCst);
        }
    }
    e.params.output = Output::Stereo;
    e.stop();
    e.render(&mut l, &mut r);
    COUNTING.store(false, SeqCst);
    let sizes: Vec<usize> = SIZES.iter().map(|s| s.load(SeqCst)).collect();
    assert_eq!(ALLOCATIONS.load(SeqCst), 0, "sizes {sizes:?}");
    assert!(rms(&l) > 0.0);
}
