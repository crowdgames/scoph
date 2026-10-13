use std::ffi::c_char;

#[repr(C)]
#[allow(nonstandard_style)]
pub enum ScophError {
    Ok,
    LoadFailed,
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_strerror(error: &ScophError) -> *const c_char {
    match error {
        ScophError::LoadFailed => c"Failed to load file",
        _ => c"",
    }
    .as_ptr()
}
