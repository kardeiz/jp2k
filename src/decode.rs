use crate::{
    err::Error,
    ffi,
    Codec, ColorSpace, ColorSpaceValue, MAX_COMPONENTS, Info,
};

use std::os::raw::c_void;
use std::ptr::null_mut;
use std::ffi::CString;
use image::DynamicImage;

#[derive(Debug, Clone, Default)]
struct DecodingArea {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

#[derive(Debug, Clone, Default)]
pub struct DecodeParams {
    default_color_space: Option<ColorSpace>,
    reduce_factor: Option<u32>,
    decoding_area: Option<DecodingArea>,
    quality_layers: Option<u32>,
}

impl DecodeParams {

    pub fn with_default_colorspace(mut self, color_space: ColorSpace) -> Self {
        self.default_color_space = Some(color_space);
        self
    }

    pub fn with_reduce_factor(mut self, reduce_factor: u32) -> Self {
        self.reduce_factor = Some(reduce_factor);
        self
    }   

    pub fn with_decoding_area(mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        self.decoding_area = Some(DecodingArea { x0, y0, x1, y1 });
        self
    }

    pub fn with_quality_layers(mut self, quality_layers: u32) -> Self {
        self.quality_layers = Some(quality_layers);
        self
    }

    fn value_for_discard_level(u: u32, discard_level: u32) -> u32 {
        let div = 1 << discard_level;
        let quot = u / div;
        let rem = u % div;
        if rem > 0 { quot + 1 } else { quot }        
    }
}

unsafe fn get_default_decoder_parameters() -> ffi::opj_dparameters {
    let jp2_dparams = ffi::opj_dparameters {
        cp_reduce: 0,
        cp_layer: 0,
        infile: [0; 4096],
        outfile: [0; 4096],
        decod_format: 0,
        cod_format: 0,
        DA_x0: 0,
        DA_x1: 0,
        DA_y0: 0,
        DA_y1: 0,
        m_verbose: 0,
        tile_index: 0,
        nb_tile_to_decode: 0,
        jpwl_correct: 0,
        jpwl_exp_comps: 0,
        jpwl_max_tiles: 0,
        flags: 0,
    };
    // ffi::opj_set_default_decoder_parameters(&mut jp2_dparams);
    jp2_dparams
}

// jp2_stream: this function will take care of deleting this at the end.
unsafe fn load_from_stream(
    jp2_stream: *mut *mut c_void,
    codec: Codec,
    params: DecodeParams,
) -> Result<DynamicImage, Error> {
    // Setup the decoder.
    let jp2_codec = ffi::opj_create_decompress(codec.to_i32());
    if jp2_codec.is_null() {
        ffi::opj_stream_destroy(jp2_stream);
        return Err(Error::FfiError("Codec instantiation failed."));
    }
    
    let mut jp2_dparams = get_default_decoder_parameters();
    
    if let Some(reduce_factor) = params.reduce_factor {
        jp2_dparams.cp_reduce = reduce_factor;
    }

    if let Some(quality_layers) = params.quality_layers {
        jp2_dparams.cp_layer = quality_layers;
    }

    if ffi::opj_setup_decoder(jp2_codec, &mut jp2_dparams) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(Error::FfiError("Setting up the decoder failed."));
    }

    let mut jp2_image: *mut ffi::opj_image = null_mut();

    // Read header.
    if ffi::opj_read_header(jp2_stream, jp2_codec, &mut jp2_image) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(Error::ReadHeader);
    }

    if let Some(DecodingArea { x0, y0, x1, y1 }) = params.decoding_area {
        if ffi::opj_set_decode_area(jp2_codec, jp2_image, x0, y0, x1, y1) != 1 {
            ffi::opj_stream_destroy(jp2_stream);
            ffi::opj_destroy_codec(jp2_codec);
            return Err(Error::FfiError("Setting up the decoding area failed."));
        }
    }

    // Decode the image.
    ffi::opj_decode(jp2_codec, jp2_stream, jp2_image);
    ffi::opj_stream_destroy(jp2_stream);

    let color_space_raw = ColorSpaceValue::from_i32((*jp2_image).color_space);
    let color_space = color_space_raw.determined().or(params.default_color_space);
    
    let color_space = if let Some(color_space) = color_space {
        color_space
    } else {
        ffi::opj_destroy_codec(jp2_codec);
        ffi::opj_image_destroy(jp2_image);
        if color_space_raw == ColorSpaceValue::Unspecified {
            return Err(Error::UnspecifiedColorSpace);
        } else {
            return Err(Error::UnknownColorSpace);
        }
    };

    let width = (*jp2_image).x1 - (*jp2_image).x0;
    let height = (*jp2_image).y1 - (*jp2_image).y0;

    let mut comps: Vec<*mut ffi::opj_image_comp> = Vec::new();
    let comps_len = (*jp2_image).numcomps;
    for i in 0..comps_len {
        comps.push((*jp2_image).comps.offset(i as isize));
    }

    if comps.len() > MAX_COMPONENTS {
        ffi::opj_destroy_codec(jp2_codec);
        ffi::opj_image_destroy(jp2_image);
        return Err(Error::TooManyComponents(comps.len()));
    }
    
    let factor = (*comps[0]).factor;
    let width = DecodeParams::value_for_discard_level(width, factor);
    let height = DecodeParams::value_for_discard_level(height, factor);

    let mut container = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let index = (x + y * width) as isize;
            let mut values = [0u8, 0, 0, 255];
            for i in 0..comps.len() {
                let data = (*comps[i]).data;
                let ivalue: u8 = *data.offset(index) as u8;
                values[i] = ivalue;
            }
            std::io::Write::write_all(&mut container, &color_space.convert_to_rgba(values))?;
        }
    }

    let buffer = image::RgbaImage::from_raw(width, height, container)
        .ok_or_else(|| Error::ImageContainerTooSmall)?;

    let image = DynamicImage::ImageRgba8(buffer);
    
    ffi::opj_destroy_codec(jp2_codec);
    ffi::opj_image_destroy(jp2_image);

    Ok(image)
}

unsafe fn info_from_stream(
    jp2_stream: *mut *mut c_void,
    codec: Codec,
) -> Result<Info, Error> {
    // Setup the decoder.
    let jp2_codec = ffi::opj_create_decompress(codec.to_i32());
    if jp2_codec.is_null() {
        ffi::opj_stream_destroy(jp2_stream);
        return Err(Error::FfiError("Codec instantiation failed."));
    }
    
    let mut jp2_dparams = get_default_decoder_parameters();
    
    jp2_dparams.flags |= ffi::OPJ_DPARAMETERS_DUMP_FLAG;
    
    if ffi::opj_setup_decoder(jp2_codec, &mut jp2_dparams) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(Error::FfiError("Setting up the decoder failed."));
    }

    let mut jp2_image: *mut ffi::opj_image = null_mut();

    // Read header.
    if ffi::opj_read_header(jp2_stream, jp2_codec, &mut jp2_image) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(Error::ReadHeader);
    }

    let width = (*jp2_image).x1 - (*jp2_image).x0;
    let height = (*jp2_image).y1 - (*jp2_image).y0;

    ffi::opj_stream_destroy(jp2_stream);
    ffi::opj_destroy_codec(jp2_codec);
    ffi::opj_image_destroy(jp2_image);

    Ok(Info { width, height })
}

pub fn load_from_memory(
    buf: &[u8],
    codec: Codec,
    decode_params: Option<DecodeParams>,
) -> Result<DynamicImage, Error> {
    
    struct SliceWithOffset<'a> {
        buf: &'a [u8],
        offset: usize,
    }

    unsafe extern "C" fn opj_stream_read_fn(
        p_buffer: *mut std::os::raw::c_void,
        p_nb_bytes: usize,
        p_user_data: *mut std::os::raw::c_void,
    ) -> usize {

        if p_buffer.is_null() { return 0; }
        
        let user_data = p_user_data as *mut SliceWithOffset;

        let len = (&*user_data).buf.len();

        let offset = (&*user_data).offset;

        let bytes_left = len - offset;

        let bytes_read = std::cmp::min(bytes_left, p_nb_bytes);

        let slice = &(&*user_data).buf[offset..offset + bytes_read];

        std::ptr::copy_nonoverlapping(slice.as_ptr(), p_buffer as *mut u8, bytes_read);

        (*user_data).offset += bytes_read;

        bytes_read
    }

    let decode_params = decode_params.unwrap_or_default();

    unsafe {
        let user_data: *mut SliceWithOffset = &mut SliceWithOffset { buf, offset: 0 };
        let stream = ffi::opj_stream_default_create(1);
        ffi::opj_stream_set_read_function(stream, Some(opj_stream_read_fn));
        ffi::opj_stream_set_user_data_length(stream, buf.len() as u64);
        ffi::opj_stream_set_user_data(stream, user_data as *mut c_void, None);
        load_from_stream(stream, codec, decode_params)
    }
}

pub fn load_from_file<T: Into<Vec<u8>>>(file_name: T, codec: Codec, decode_params: Option<DecodeParams>) -> Result<DynamicImage, Error> {
    let decode_params = decode_params.unwrap_or_default();
    let file_name = CString::new(file_name.into())?;
    unsafe {
        let jp2_stream = ffi::opj_stream_create_default_file_stream(file_name.as_ptr(), 1);
        load_from_stream(jp2_stream, codec, decode_params)
    }
}

pub fn info<T: Into<Vec<u8>>>(file_name: T, codec: Codec) -> Result<Info, Error> {
    let file_name = CString::new(file_name.into())?;
    unsafe {
        let jp2_stream = ffi::opj_stream_create_default_file_stream(file_name.as_ptr(), 1);
        info_from_stream(jp2_stream, codec)
    }
}