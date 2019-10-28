extern crate bindgen;

use std::env;
use std::path::PathBuf;

#[cfg(feature = "docs-rs")]
fn main() {}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let lib = pkg_config::probe_library("libopenjp2").expect("Could not find `libopenjp2`");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut builder = bindgen::Builder::default();

    for path in &lib.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.display()));
    }

    let bindings = builder
        .header_contents("wrapper.h", "#include \"openjpeg.h\"")
        .clang_arg("-fno-inline-functions")
        .derive_debug(true)
        .impl_debug(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .rustfmt_bindings(true)
        .generate()
        .unwrap();

    // bindings.write_to_file("src/ffi.ref.rs").unwrap();

    bindings.write_to_file(out_path.join("bindings.rs")).unwrap();
}
