use std::ffi::{CStr, c_char, c_uint};

use crate::{ScophError, ScophIntrp};

/// Provides the size in cells of a layer on the game's board. Will return a
/// [`ScophError::MissingLayerOnSrc`] if the layer doesn't exist on the board.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_board_layer_size(
    intrp: *const ScophIntrp,
    layer_name: *const c_char,
    width: *mut c_uint,
    height: *mut c_uint,
) -> ScophError {
    let intrp = unsafe { &(*intrp).0 };
    let layer_name = unsafe { CStr::from_ptr(layer_name) };

    let Some((w, h)) =
        intrp.board_layer_size(layer_name.to_str().expect("should be a valid utf8 string"))
    else {
        return ScophError::MissingLayerOnSrc;
    };
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
