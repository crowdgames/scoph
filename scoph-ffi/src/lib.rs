use std::ffi::{CStr, OsStr, c_char};

/// cbindgen:ignore
use scoph::interpreter::{BehaviorTree, Interpreter};

pub struct ScophIntrp(Interpreter);

mod board;
mod error;

use error::ScophError;

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_load(intrp: *mut *mut ScophIntrp, path: *const c_char) -> ScophError {
    let path = unsafe { CStr::from_ptr(path) };
    let path = unsafe { OsStr::from_encoded_bytes_unchecked(path.to_bytes()) };

    let bt = match BehaviorTree::load_from_json_file(path) {
        Ok(bt) => bt,
        Err(_) => return ScophError::LoadFailed,
    };

    let new = ScophIntrp(Interpreter::new(bt));
    unsafe {
        *intrp = Box::into_raw(Box::new(new));
    }

    ScophError::Ok
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_free(intrp: *mut *mut ScophIntrp) {
    unsafe { drop(Box::from_raw(*intrp)) }
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
    let intrp = unsafe { &mut (*intrp).0 };

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
