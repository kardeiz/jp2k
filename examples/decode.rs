fn main() {


    let bytes = include_bytes!("/mnt/c/projects/iiif-server/cache/remote/45928.jp2");


    let codec = jp2k::Codec::create(jp2k::CODEC_FORMAT::OPJ_CODEC_JP2).unwrap();
    let stream = jp2k::Stream::from_bytes(bytes).unwrap();


    let jp2k::ImageBuffer { buffer, width, height, num_bands } = jp2k::ImageBuffer::build(
        jp2k::DecodeParams::default().with_reduce_factor(3), codec, stream).unwrap();

    let img = rips::Image::from_memory(
        buffer,
        width as i32,
        height as i32,
        num_bands as i32,
        rips::VipsBandFormat::VIPS_FORMAT_UCHAR,
    ).unwrap();

    img.write_to_file("test.png").unwrap();


}
