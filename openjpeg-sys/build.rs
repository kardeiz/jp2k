// debugging instructions: run cargo with -vv to get all output.

extern crate bindgen;
extern crate cmake;
extern crate cc;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;


#[cfg(unix)]
mod supported_platform {
    pub fn check() {}
}

fn main() {
    supported_platform::check();

    if !Path::new("libopenjpeg/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    // Unset DESTDIR or libopenjp2.a ends up in it and cargo won't find it.
    env::remove_var("DESTDIR");
    let mut cfg = cmake::Config::new("libopenjpeg");
    let dst = cfg.define("BUILD_SHARED_LIBS", "OFF").build();

    println!("cargo:rustc-link-search=native={}/build/bin", dst.display());
    println!("cargo:rustc-link-lib=static=openjp2");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_path.join("include/openjpeg-2.3");
    let library_dir = out_path.join("build/bin");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // issue: https://github.com/rust-lang-nursery/rust-bindgen/issues/348
        .clang_arg("-fno-inline-functions")
        .clang_arg(format!("-I{}", include_dir.display()))
        .generate()
        .unwrap();

    // Write bindings to $OUT_DIR/bindings.rs
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
