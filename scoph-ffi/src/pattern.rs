use core::ffi::{CStr, c_char, c_uint};

///cbindgen:ignore
use scoph::common::TRRBTPattern;

use crate::error::ScophError;

pub struct Pattern(pub *const TRRBTPattern);

#[unsafe(no_mangle)]
pub extern "C" fn scoph_pattern_size(pat: Pattern, width: *mut c_uint, height: *mut c_uint) {
    let pat = unsafe { &(*(pat).0) };

    unsafe {
        let (w, h) = pat.pattern_size();
        *width = w as c_uint;
        *height = h as c_uint;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_pattern_cell(
    pat: Pattern,
    x: c_uint,
    y: c_uint,
    layer_name: *const c_char,
    dst: *mut *const c_char,
) -> ScophError {
    let pat = unsafe { &(*(pat).0) };
    let layer_name = unsafe { CStr::from_ptr(layer_name).to_str().unwrap() };

    match pat.get_cell_c(x as usize, y as usize, layer_name) {
        Ok(c) => {
            unsafe {
                *dst = c;
            }
            ScophError::Ok
        }
        Err(e) => e.into(),
    }
}
