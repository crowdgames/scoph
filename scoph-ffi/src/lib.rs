use std::{
    ffi::{CStr, OsStr, c_char},
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
    Failed,
    Finished,
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_step(intrp: *mut ScophIntrp, res: *mut ScophIntrpResult) -> ScophError {
    let mut intrp = unsafe {
        &mut *intrp
    };

    if let Err(i) = intrp.step() {

    }

    ScophError::Ok
}
