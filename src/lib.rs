extern crate image;
extern crate libc;

pub use self::image::{ImageResult, DynamicImage};

pub mod decode;
pub mod error;

/*
pub fn load_from_memory(buffer: &[u8], codec: Codec) -> ImageResult<DynamicImage> {
    

    unimplemented!()
}

*/
