use std::ffi::{CStr, c_char, c_uint};

/// cbindgen:ignore
use scoph::common::TRRBTPattern;

use crate::{ScophError, ScophIntrp};

pub struct Pattern<'a>(&'a TRRBTPattern);

/// Gives the size of a pattern's dimensions.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_pattern_size(
    pat: *const Pattern,
    pat_width: *mut c_uint,
    pat_height: *mut c_uint,
) {
    let (w, h) = unsafe { (*pat).0.pattern_size() };

    unsafe {
        *pat_width = w as c_uint;
        *pat_height = h as c_uint;
    }
}

/// Gives the cell at the specified layer and position.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_pattern_cell(
    pat: *const Pattern,
    layer_name: *const c_char,
    x: c_uint,
    y: c_uint,
    dst: *mut *const c_char,
) -> ScophError {
    let pat = unsafe { (*pat).0 };

    match pat.get_cell_c(x as usize, y as usize, unsafe {
        CStr::from_ptr(layer_name).to_str().unwrap()
    }) {
        Ok(cell) => unsafe {
            *dst = cell;
            ScophError::Ok
        },
        Err(e) => e.into(),
    }
}

/// Provides the size in cells of a layer on the game's board. Will return a
/// [`ScophError::MissingLayerOnSrc`] if the layer doesn't exist on the board.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_board_size(
    intrp: *const ScophIntrp,
    width: *mut c_uint,
    height: *mut c_uint,
) -> ScophError {
    let intrp = unsafe { &(*intrp).0 };

    let (w, h) = intrp.board_size();
    unsafe {
        *width = w as c_uint;
        *height = h as c_uint;
    }
    ScophError::Ok
}

/// Gets the cell of the grid at the specified layer and coordinates. Returns a
/// [ScophError::MissingLayerSrc](crate::error::ScophError)
#[unsafe(no_mangle)]
pub extern "C" fn scoph_get_board_cell(
    intrp: *const ScophIntrp,
    layer_name: *const c_char,
    dest: *mut *const c_char,
    x: c_uint,
    y: c_uint,
) -> ScophError {
    let intrp = unsafe { &(*intrp).0 };

    struct UnsafeVisitor<'a> {
        layer_name: &'a CStr,
        x: usize,
        y: usize,
        dest: *mut *const c_char,
        error: ScophError,
    }

    impl<'a> scoph::interpreter::Visitor for UnsafeVisitor<'a> {
        fn view_board(&mut self, board: &scoph::common::TRRBTPattern) {
            match board.get_cell_c(
                self.x,
                self.y,
                &self
                    .layer_name
                    .to_str()
                    .expect("Should be a valid UTF-8 layer name"),
            ) {
                Ok(cell) => unsafe {
                    *self.dest = cell;
                },
                Err(e) => {
                    self.error = e.into();
                }
            }
        }
    }

    let mut uv = UnsafeVisitor {
        layer_name: unsafe { CStr::from_ptr(layer_name) },
        x: x as usize,
        y: y as usize,
        dest,
        error: ScophError::Ok,
    };
    intrp.view_board(&mut uv);

    uv.error
}
