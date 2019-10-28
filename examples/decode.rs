fn main() {
    let bytes = include_bytes!("rust-logo-512x512-blk.jp2");

    let codec = jp2k::Codec::jp2();
    let stream = jp2k::Stream::from_bytes(bytes).unwrap();
    // let stream = jp2k::Stream::from_file("/mnt/c/projects/iiif-server/cache/remote/322930.jp2").unwrap();

    let jp2k::ImageBuffer { buffer, width, height, num_bands } = jp2k::ImageBuffer::build(
        codec,
        stream,
        jp2k::DecodeParams::default().with_reduce_factor(1),
    )
    .unwrap();

    let img = rips::Image::from_memory(
        buffer,
        width as i32,
        height as i32,
        num_bands as i32,
        rips::VipsBandFormat::VIPS_FORMAT_UCHAR,
    )
    .unwrap();

    img.write_to_file("examples/output/test.png").unwrap();
}
