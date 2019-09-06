pub use image::DynamicImage;

mod ffi;

pub mod err;

mod decode;

pub use decode::{
    DecodeParams,
    load_from_memory,
    load_from_file,
    info
};

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
            ColorSpaceValue::Unknown(_) |
            ColorSpaceValue::Unspecified => None,
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

