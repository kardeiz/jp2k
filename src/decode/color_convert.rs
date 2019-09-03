use image::Rgba;

/// This is the type only describing the actual ColorSpaces and doesn't allow for the `Unknown` and
/// `Unspecified` variant.
#[allow(dead_code)] // TODO: remove
#[derive(Clone, Debug)]
pub enum ColorSpace {
    CMYK,
    EYCC,
    GRAY,
    SRGB,
    SYCC,
}

/// This is a type used for decoding the color space type as provided by the C API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorSpaceValue {
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
}

/// The maximum number of components used in any pixel encoding.
pub const MAX_COMPONENTS: usize = 4;
type ArrComponents = [u8; MAX_COMPONENTS];

impl ColorSpace {
    pub fn convert_to_rgba(&self, source: ArrComponents) -> Rgba<u8> {
        let result: [u8; 4] = match *self {
            ColorSpace::SRGB => source,
            ColorSpace::GRAY => [source[0], source[0], source[0], 255],
            _ => unimplemented!(),
        };

        Rgba { data: result }
    }
}
