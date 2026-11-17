//! Compiles the vendored C++ engines (see third_party/) into static libraries.

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../third_party");
    let shim = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shim");
    println!("cargo:rerun-if-changed=shim");
    println!("cargo:rerun-if-changed=../../third_party");

    let base = |std: &str| {
        let mut b = cc::Build::new();
        b.cpp(true)
            .std(std)
            .opt_level(3)
            .flag_if_supported("-fno-math-errno")
            .flag_if_supported("-Wno-deprecated")
            .flag_if_supported("-Wno-deprecated-declarations")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-unused-variable")
            .flag_if_supported("-Wno-unused-but-set-variable")
            .flag_if_supported("-Wno-sign-compare")
            .flag_if_supported("-Wno-misleading-indentation")
            .flag_if_supported("-Wno-dynamic-exception-spec")
            .flag_if_supported("-Wno-unused-private-field")
            .flag_if_supported("-Wno-unused-function")
            .flag_if_supported("-Wno-reorder")
            .flag_if_supported("-Wno-class-memaccess")
            .flag_if_supported("-Wno-ignored-qualifiers")
            .warnings(false)
            // M_PI and friends on Windows
            .define("_USE_MATH_DEFINES", None);
        b
    };

    // Open303
    let o3 = root.join("open303");
    cc::Build::new()
        .file(o3.join("fft4g.c"))
        .opt_level(3)
        .warnings(false)
        .compile("f303_fft4g");
    let mut b = base("c++14");
    b.include(&o3);
    for entry in std::fs::read_dir(&o3).expect("open303 sources") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "cpp") {
            b.file(path);
        }
    }
    b.file(shim.join("open303.cpp")).compile("f303_open303");

    // freeverb3 and the Dragonfly engines (dynamic exception specifications: C++14)
    let fv = root.join("freeverb3");
    let mut b = base("c++14");
    b.include(&fv).include(&shim).define("LIBFV3_FLOAT", None);
    for name in [
        "allpass",
        "biquad",
        "comb",
        "delay",
        "delayline",
        "earlyref",
        "efilter",
        "nrev",
        "nrevb",
        "progenitor",
        "progenitor2",
        "revbase",
        "slot",
        "strev",
        "utils",
        "zrev",
        "zrev2",
        "fv3_rt_pool",
    ] {
        b.file(fv.join("freeverb").join(format!("{name}.cpp")));
    }
    b.file(shim.join("dragonfly.cpp")).compile("f303_dragonfly");

    // Aether
    let mut b = base("c++20");
    b.include(root.join("aether/src"))
        .include(&root)
        .include(&shim)
        .file(root.join("aether/src/DSP/aether_dsp.cpp"))
        .file(shim.join("aether.cpp"))
        .compile("f303_aether");

    // CloudSeed (CloudReverb)
    let cs = root.join("cloudseed");
    let mut b = base("c++17");
    b.include(&cs).include(&shim);
    for f in [
        "FastSin.cpp",
        "AudioLib/Biquad.cpp",
        "AudioLib/ShaRandom.cpp",
        "AudioLib/ValueTables.cpp",
        "Utils/Sha256.cpp",
    ] {
        b.file(cs.join(f));
    }
    b.file(shim.join("cloud.cpp"))
        .file(shim.join("reverb.cpp"))
        .compile("f303_cloud");
}
