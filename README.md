# jp2k


## Rust bindings to OpenJPEG

Supports loading JPEG2000 images into `image::DynamicImage`.Rust

Forked from https://framagit.org/leoschwarz/jpeg2000-rust before its GPL-v3 relicensing, with some additional features:

* Specify decoding area and quality layers in addition to reduction factor
* Improved OpenJPEG -> DynamicImage loading process
* Get basic metadata from JPEG2000 headings
* Docs (albeit minimal ones)

This library brings its own libopenjpeg, which is statically linked. If you just need raw FFI bindings, see
[openjpeg2-sys](https://crates.io/crates/openjpeg2-sys) or [openjpeg-sys](https://crates.io/crates/openjpeg-sys).


### Usage

```rust
fn main() {
    let bytes = include_bytes!("./rust-logo-512x512-blk.jp2");

    let jp2k::Image(img) = jp2k::Image::from_bytes(
        bytes,
        jp2k::Codec::JP2,
        Some(jp2k::DecodeParams::default().with_decoding_area(0, 0, 256, 256))
    )
    .unwrap();

    let mut output = std::path::Path::new("examples/output/result.png");
    let _ = img.save(&mut output);
}
```

### Original warnings and license statement

#### Warning
Please be advised that using C code means this crate is likely vulnerable to various memory exploits, e.g. see [http://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2016-8332](CVE-2016-8332) for an actual example from the past.

As soon as someone writes an efficient JPEG2000 decoder in pure Rust you should probably switch over to that.

#### License
You can use the Rust code in the directories `src` and `openjp2-sys/src` under the terms of either the MIT license (`LICENSE-MIT` file) or the Apache license (`LICENSE-APACHE` file). Please note that this will link statically to OpenJPEG, which has its own license which you can find at `openjpeg-sys/libopenjpeg/LICENSE` (you might have to check out the git submodule first).

License: MIT OR Apache-2.0
