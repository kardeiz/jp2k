fn main() {
    // TODO this doesn't work as of now.
    //let mut buffer = include_bytes!("./rust-logo-512x512-blk.jp2").to_vec();
    //let img = jpeg2000::decode::load_from_memory(&mut buffer[..], Codec::JP2).unwrap();

    let info = jp2k::decode::info(
        "./examples/rust-logo-512x512-blk.jp2",
        //CString::new("./examples/relax.jp2").unwrap(),
        jp2k::decode::Codec::JP2,
    ).unwrap();

    // println!("{:?}", (width, height));

    // let img = jp2k::decode::load_from_file(
    //     "./examples/rust-logo-512x512-blk.jp2",
    //     //CString::new("./examples/relax.jp2").unwrap(),
    //     jp2k::decode::Codec::JP2,
    //     Some(jp2k::decode::DecodeParams { 
    //         default_colorspace: None, 
    //         reduce_factor: Some(1), 
    //         decoding_area: Some(jp2k::decode::DecodingArea { x0: 0, y0: 0, x1: 100, y1: 100 })
    //     })
    // ).unwrap();




    let bytes = include_bytes!("./rust-logo-512x512-blk.jp2");

    let img = jp2k::decode::load_from_memory(
        bytes,
        jp2k::decode::Codec::JP2,
        Some(jp2k::decode::DecodeParams { 
            default_colorspace: None, 
            reduce_factor: None, 
            quality_layers: None,
            decoding_area: Some(jp2k::decode::DecodingArea { x0: 0, y0: 0, x1: 256, y1: 256 })
        })
    ).unwrap();

    let mut output = std::path::Path::new("examples/output/result.png");
    let _ = img.save(&mut output);
}
