extern crate cmake;
extern crate cxx_build;
use cmake::Config;

fn main() {
    // 1. Build verona-rt to fetch dependencies (snmalloc)
    // We use external/verona-rt as the source directory.
    let verona_rt_dst = Config::new("external/verona-rt")
        .define("VERONA_RT_ONLY_HEADER_LIBRARY", "ON")
        .build();

    // 2. Find snmalloc include path
    // cmake crate builds in <OUT_DIR>.
    // FetchContent usually places sources in <build_dir>/_deps/snmalloc-src
    let snmalloc_path = verona_rt_dst.join("build/_deps/snmalloc-src/src");

    println!(
        "cargo:rustc-link-search=native={}",
        verona_rt_dst.join("lib").display()
    );

    // 3. Configure cxx_build
    let mut build = cxx_build::bridge("src/verona_stubs.rs");
    build
        .file("libverona/verona.cc")
        .include("external/verona-rt/src/rt")
        .include(snmalloc_path)
        .include(".")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-mcx16")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-sign-compare");

    if cfg!(target_os = "linux") {
        build.flag_if_supported("-pthread");
    }

    build.compile("verona_bridge");

    println!("cargo:rerun-if-changed=src/verona_stubs.rs");
    println!("cargo:rerun-if-changed=libverona/verona.cc");
    println!("cargo:rerun-if-changed=external/verona-rt");

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        println!("cargo:rustc-link-lib=atomic");
    }
}
