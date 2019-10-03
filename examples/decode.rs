// fn main() {
//     let bytes = include_bytes!("./rust-logo-512x512-blk.jp2");

//     let jp2k::Image(img) = jp2k::Image::from_bytes(
//         bytes,
//         jp2k::Codec::JP2,
//         Some(jp2k::DecodeParams::default().with_decoding_area(0, 0, 256, 256))
//     )
//     .unwrap();

//     let mut output = std::path::Path::new("examples/output/result.png");
//     let _ = img.save(&mut output);
// }

fn main() {
    // let bytes = include_bytes!("/mnt/c/projects/jp2k/examples/rust-logo-512x512-blk.jp2");

    // let bytes: std::sync::Arc<[u8]> = bytes.to_vec().into();

    let bytes = include_bytes!("/mnt/c/projects/iiif-server/cache/remote/45928.jp2");


    let codec = jp2k::Codec::create(jp2k::CODEC_FORMAT::OPJ_CODEC_JP2).unwrap();
    let stream = jp2k::Stream::from_bytes(bytes).unwrap();

    // let y = jp2k::DecodeParams::default();
    // // let stream = jp2k::Stream::from_file("/mnt/c/projects/jp2k/examples/rust-logo-512x512-blk.jp2").unwrap();

    // // let img = jp2k::Image::info_only(codec, stream).unwrap();


    // let x  =jp2k::Image::info_on/**/ly(codec, stream).unwrap();


    // println!("{:?}", &y);


    let jp2k::ImageBuffer { buffer, width, height, num_bands } = jp2k::ImageBuffer::build(
        jp2k::DecodeParams::default().with_reduce_factor(3), codec, stream).unwrap();
    // println!("{:?}", bytes.len());

    let img = rips::Image::from_memory(
        buffer,
        width as i32,
        height as i32,
        num_bands as i32,
        rips::VipsBandFormat::VIPS_FORMAT_UCHAR,
    ).unwrap();

    img.write_to_file("test.png").unwrap();

    // println!("{:?}", unsafe { (&*img.0) });


}
