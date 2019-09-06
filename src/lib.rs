/*!

# Rust bindings to OpenJPEG

Supports loading JPEG2000 images into `image::DynamicImage`.Rust

Forked from https://framagit.org/leoschwarz/jpeg2000-rust before its GPL-v3 relicensing, with some additional features:

* Specify decoding area and quality layers in addition to reduction factor
* Improved OpenJPEG -> DynamicImage loading process
* Get basic metadata from JPEG2000 headings
* Docs (albeit minimal ones)

This library brings its own libopenjpeg, which is statically linked. If you just need raw FFI bindings, see 
[openjpeg2-sys](https://crates.io/crates/openjpeg2-sys) or [openjpeg-sys](https://crates.io/crates/openjpeg-sys).


## Usage

```rust,no_run
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

## Original warnings and license statement

### Warning
Please be advised that using C code means this crate is likely vulnerable to various memory exploits, e.g. see [http://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2016-8332](CVE-2016-8332) for an actual example from the past.

As soon as someone writes an efficient JPEG2000 decoder in pure Rust you should probably switch over to that.

### License
You can use the Rust code in the directories `src` and `openjp2-sys/src` under the terms of either the MIT license (`LICENSE-MIT` file) or the Apache license (`LICENSE-APACHE` file). Please note that this will link statically to OpenJPEG, which has its own license which you can find at `openjpeg-sys/libopenjpeg/LICENSE` (you might have to check out the git submodule first).
*/

extern crate image;

pub mod err;

mod ffi;
mod decode;

pub use decode::DecodeParams;

use std::path::Path;

/// Wrapper around `image::DynamicImage`
pub struct Image(pub image::DynamicImage);

impl Image {
    /// Return the inner `image::DynamicImage`
    pub fn into_inner(self) -> image::DynamicImage { self.0 }

    /// Load image from file path
    pub fn from_file<P: AsRef<Path>>(
        file_name: P,
        codec: Codec,
        decode_params: Option<DecodeParams>,
    ) -> err::Result<Self> {
        Ok(Image(decode::load_from_file(file_name.as_ref().display().to_string(), codec, decode_params)?))
    }

    /// Load image from bytes
    pub fn from_bytes(
        buf: &[u8],
        codec: Codec,
        decode_params: Option<DecodeParams>,
    ) -> err::Result<Self> {
        Ok(Image(decode::load_from_bytes(buf, codec, decode_params)?))
    }
}

/// This is the type only describing the actual ColorSpaces and doesn't allow for the `Unknown` and
/// `Unspecified` variant.
#[derive(Clone, Debug)]
pub enum ColorSpace {
    CMYK,
    EYCC,
    GRAY,
    SRGB,
    SYCC,
}

/// File type
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Codec {
    J2K,
    JP2,
    JPP,
    JPT,
    JPX,
}

impl Codec {
    fn to_i32(&self) -> i32 {
        match *self {
            Codec::J2K => ffi::CODEC_FORMAT_OPJ_CODEC_J2K,
            Codec::JP2 => ffi::CODEC_FORMAT_OPJ_CODEC_JP2,
            Codec::JPP => ffi::CODEC_FORMAT_OPJ_CODEC_JPP,
            Codec::JPT => ffi::CODEC_FORMAT_OPJ_CODEC_JPT,
            Codec::JPX => ffi::CODEC_FORMAT_OPJ_CODEC_JPX,
        }
    }
}

/// Information about a JPEG-2000 file
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub width: u32,
    pub height: u32,
}

impl Info {

    /// Load info from file path
    pub fn from_file<P: AsRef<Path>>(file_name: P, codec: Codec) -> err::Result<Self> {
        decode::info_from_file(file_name.as_ref().display().to_string(), codec)
    }

    /// Load info from bytes
    pub fn from_bytes(buf: &[u8], codec: Codec) -> err::Result<Self> {
        decode::info_from_bytes(buf, codec)
    }
}

/// This is a type used for decoding the color space type as provided by the C API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ColorSpaceValue {
    CMYK,
    EYCC,
    GRAY,
    SRGB,
    SYCC,
    Unknown(i32),
    Unspecified,
}

impl ColorSpaceValue {
    pub fn determined(&self) -> Option<ColorSpace> {
        match *self {
            ColorSpaceValue::CMYK => Some(ColorSpace::CMYK),
            ColorSpaceValue::EYCC => Some(ColorSpace::EYCC),
            ColorSpaceValue::GRAY => Some(ColorSpace::GRAY),
            ColorSpaceValue::SRGB => Some(ColorSpace::SRGB),
            ColorSpaceValue::SYCC => Some(ColorSpace::SYCC),
            ColorSpaceValue::Unknown(_) | ColorSpaceValue::Unspecified => None,
        }
    }

    pub fn from_i32(val: i32) -> Self {
        match val {
            ffi::COLOR_SPACE_OPJ_CLRSPC_CMYK => ColorSpaceValue::CMYK,
            ffi::COLOR_SPACE_OPJ_CLRSPC_EYCC => ColorSpaceValue::EYCC,
            ffi::COLOR_SPACE_OPJ_CLRSPC_GRAY => ColorSpaceValue::GRAY,
            ffi::COLOR_SPACE_OPJ_CLRSPC_SRGB => ColorSpaceValue::SRGB,
            ffi::COLOR_SPACE_OPJ_CLRSPC_SYCC => ColorSpaceValue::SYCC,
            ffi::COLOR_SPACE_OPJ_CLRSPC_UNKNOWN => ColorSpaceValue::Unknown(val),
            ffi::COLOR_SPACE_OPJ_CLRSPC_UNSPECIFIED => ColorSpaceValue::Unspecified,
            _ => ColorSpaceValue::Unknown(val),
        }
    }
}

pub(crate) const MAX_COMPONENTS: usize = 4;

impl ColorSpace {
    pub(crate) fn convert_to_rgba(&self, source: [u8; MAX_COMPONENTS]) -> [u8; MAX_COMPONENTS] {
        let result = match *self {
            ColorSpace::SRGB => source,
            ColorSpace::GRAY => [source[0], source[0], source[0], 255],
            _ => unimplemented!(),
        };

        result
    }
}
