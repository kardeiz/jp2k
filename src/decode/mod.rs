use openjpeg_sys as ffi;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::ffi::CString;
use image::{DynamicImage, GenericImage};

use crate::error::DecodeError;

mod color_convert;
use self::color_convert::{ColorSpace, ColorSpaceValue};

// TODO: in the future remove all mem::uninitialized and mem::zeroed from this crate,
// I use these as it's quicker to get something running, but they might end up sneaking UB
// into our code.

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


impl ColorSpaceValue {
    fn from_i32(val: i32) -> Self {
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

/*
extern "C" fn quiet_callback(_: *const c_char, _: *mut c_void) {}
*/

unsafe fn get_default_decoder_parameters() -> ffi::opj_dparameters {
    let mut jp2_dparams = ffi::opj_dparameters {
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
    ffi::opj_set_default_decoder_parameters(&mut jp2_dparams);
    jp2_dparams
}

// jp2_stream: this function will take care of deleting this at the end.
unsafe fn load_from_stream(
    jp2_stream: *mut *mut c_void,
    codec: Codec,
) -> Result<DynamicImage, DecodeError> {
    // Setup the decoder.
    let jp2_codec = ffi::opj_create_decompress(codec.to_i32());
    if jp2_codec.is_null() {
        ffi::opj_stream_destroy(jp2_stream);
        return Err(DecodeError::FfiError("Codec instantiation failed."));
    }
    let mut jp2_dparams = get_default_decoder_parameters();
    if ffi::opj_setup_decoder(jp2_codec, &mut jp2_dparams) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(DecodeError::FfiError("Setting up the decoder failed."));
    }

    /*
    // Set quiet callbacks.
    ffi::opj_set_info_handler(jp2_codec, Some(quiet_callback), null_mut());
    ffi::opj_set_warning_handler(jp2_codec, Some(quiet_callback), null_mut());
    ffi::opj_set_error_handler(jp2_codec, Some(quiet_callback), null_mut());
    */

    // Read header.
    let mut jp2_image: *mut ffi::opj_image = &mut ffi::opj_image {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        numcomps: 0,
        color_space: 0,
        comps: null_mut(),
        icc_profile_buf: null_mut(),
        icc_profile_len: 0,
    };
    if ffi::opj_read_header(jp2_stream, jp2_codec, &mut jp2_image) != 1 {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        return Err(DecodeError::ReadHeader);
    }

    // Decode the image.
    ffi::opj_decode(jp2_codec, jp2_stream, jp2_image);

    let color_space_raw = ColorSpaceValue::from_i32((*jp2_image).color_space);
    let color_space = color_space_raw.determined();
    if color_space.is_none() {
        // TODO: how to deal with unspecified color space?
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        ffi::opj_image_destroy(jp2_image);
        if color_space_raw == ColorSpaceValue::Unspecified {
            return Err(DecodeError::UnspecifiedColorSpace);
        } else {
            return Err(DecodeError::UnknownColorSpace);
        }
    }
    let color_space: ColorSpace = color_space.unwrap();

    let width = (*jp2_image).x1 - (*jp2_image).x0;
    let height = (*jp2_image).y1 - (*jp2_image).y0;

    //println!("width: {}, height: {}", width, height);

    //println!("color_space: {:?}", color_space);
    //println!("icc_profile_len: {}", (*jp2_image).icc_profile_len);
    //println!("numcomps: {}", (*jp2_image).numcomps);

    let mut comps: Vec<*mut ffi::opj_image_comp> = Vec::new();
    let comps_len = (*jp2_image).numcomps;
    for i in 0..comps_len {
        comps.push((*jp2_image).comps.offset(i as isize));
    }
    //println!("comps.len() = {}", comps.len());

    //let mut jp2_info: *mut c_void = mem::zeroed();
    //ffi::opj_get_cstr_info(&mut jp2_info);

    if comps.len() > color_convert::MAX_COMPONENTS {
        ffi::opj_stream_destroy(jp2_stream);
        ffi::opj_destroy_codec(jp2_codec);
        ffi::opj_image_destroy(jp2_image);
        return Err(DecodeError::TooManyComponents(comps.len()));
    }
    let mut image = DynamicImage::new_rgba8(width, height);

    // Copy the pixels.
    for y in 0..height {
        for x in 0..width {
            let index = (x + y * width) as isize;

            let mut values = [0u8, 0, 0, 0];
            for i in 0..comps.len() {
                let data = (*comps[i]).data;
                //assert!((*comps[i]).sgnd == 0); // TODO signed numbers?!
                let ivalue: u8 = *data.offset(index) as u8;
                values[i] = ivalue;
            }

            image.unsafe_put_pixel(x, y, color_space.convert_to_rgba(values))
        }
    }

    ffi::opj_stream_destroy(jp2_stream);
    ffi::opj_destroy_codec(jp2_codec);
    ffi::opj_image_destroy(jp2_image);

    Ok(image)
}

/*
// TODO: Apparently this is still missing https://github.com/uclouvain/openjpeg/issues/972
pub fn load_from_memory(buf: &mut [u8], codec: Codec) -> Result<DynamicImage, DecodeError> {
    unsafe {
        let jp2_stream = ffi::opj_stream_create(buf.len(), 1);
        //ffi::opj_stream_set_user_data_length(jp2_stream, buf.len() as u64);
        ffi::opj_stream_set_user_data(jp2_stream, buf.as_mut_ptr() as *mut c_void, None);
        ffi::opj_stream_set_read_function(jp2_stream, Some(full_read_buf));
        load_from_stream(jp2_stream, codec)
    }
}
*/

// TODO: docs
pub fn load_from_file(fname: CString, codec: Codec) -> Result<DynamicImage, DecodeError> {
    unsafe {
        let jp2_stream = ffi::opj_stream_create_default_file_stream(fname.as_ptr(), 1);
        load_from_stream(jp2_stream, codec)
    }
}
