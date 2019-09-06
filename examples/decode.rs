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
