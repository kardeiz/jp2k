extern crate jpeg2000;
extern crate image;

use std::fs::File;
use jpeg2000::decode::Codec;
// TODO this shouldn't be needed here.
use std::ffi::CString;

fn main() {
    // TODO this doesn't work as of now.
    //let mut buffer = include_bytes!("./rust-logo-512x512-blk.jp2").to_vec();
    //let img = jpeg2000::decode::load_from_memory(&mut buffer[..], Codec::JP2).unwrap();

    let img = jpeg2000::decode::load_from_file(
        CString::new("./examples/rust-logo-512x512-blk.jp2").unwrap(),
        //CString::new("./examples/relax.jp2").unwrap(),
        Codec::JP2,
    ).unwrap();

    let mut output = File::create("result.png").unwrap();
    let _ = img.save(&mut output, image::ImageFormat::PNG);
}
