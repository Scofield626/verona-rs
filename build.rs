extern crate cmake;
use cmake::Config;

fn main() {
    let dst = Config::new("libverona").build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=verona");

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=atomic");
    }

    #[cfg(target_arch = "aarch64")]
    {
        println!("cargo:rustc-link-lib=c++");
    }
}
