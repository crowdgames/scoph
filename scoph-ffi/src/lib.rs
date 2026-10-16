use std::{
    ffi::{CStr, OsStr, c_char, c_uint},
    os::unix::ffi::OsStrExt,
};

use scoph::interpreter::BehaviorTree;
pub use scoph::interpreter::Interpreter as ScophIntrp;

mod error;

pub use error::{ScophError, scoph_strerror};

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_load(intrp: *mut ScophIntrp, path: *const c_char) -> ScophError {
    let path = unsafe { CStr::from_ptr(path) };
    let path = OsStr::from_bytes(path.to_bytes());

    let bt = match BehaviorTree::load_from_json_file(path) {
        Ok(bt) => bt,
        Err(_) => return ScophError::LoadFailed,
    };

    unsafe {
        *intrp = ScophIntrp::new(bt);
    }

    ScophError::Ok
}

#[repr(C)]
#[derive(Default)]
pub enum ScophIntrpResult {
    #[default]
    Unstarted,
    InProgress,
    Failed,
    Finished,
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_step(
    intrp: *mut ScophIntrp,
    res: *mut ScophIntrpResult,
) -> ScophError {
    let intrp = unsafe { &mut *intrp };

    match intrp.step() {
        Ok(finished) => unsafe {
            *res = if finished {
                ScophIntrpResult::Finished
            } else {
                ScophIntrpResult::InProgress
            }
        },
        Err(i) => {
            unsafe {
                *res = ScophIntrpResult::Failed;
            }
            return i.into();
        }
    }

    ScophError::Ok
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_view_board_cell(
    intrp: *const ScophIntrp,
    layer_name: *const c_char,
    x: c_uint,
    y: c_uint,
    dest: *mut c_char,
    bytes_read: *mut c_uint,
) -> ScophError {
    let intrp = unsafe { &*intrp };

    struct UnsafeVisitor<'a> {
        layer_name: &'a CStr,
        x: usize,
        y: usize,
        dest: *mut c_char,
        bytes_read: *mut c_uint,
        error: Option<ScophError>,
    }

    impl<'a> scoph::interpreter::Visitor for UnsafeVisitor<'a> {
        fn view_board(&mut self, board: &scoph::common::TRRBTPattern) {
            match board.get_cell(
                self.x,
                self.y,
                &self
                    .layer_name
                    .to_str()
                    .expect("Should be a valid UTF-8 layer name"),
            ) {
                Ok(cell) => unsafe {
                    *self.bytes_read = cell.len() as c_uint;
                    std::ptr::copy_nonoverlapping(
                        cell.as_ptr(),
                        self.dest as *mut u8,
                        self.bytes_read as usize,
                    );
                },
                Err(e) => {
                    self.error = Some(e.into());
                }
            }
        }
    }

    let mut uv = UnsafeVisitor {
        layer_name: unsafe { CStr::from_ptr(layer_name) },
        x: x as usize,
        y: y as usize,
        dest,
        bytes_read,
        error: None,
    };
    intrp.view_board(&mut uv);

    uv.error.unwrap_or(ScophError::Ok)
}
