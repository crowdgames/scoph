use std::ffi::c_char;

use scoph::interpreter::IError;

#[repr(C)]
#[allow(nonstandard_style)]
pub enum ScophError {
    Ok,
    LoadFailed,
    MissingChild,
    MissingLayerOnSrc,
    MissingLayerOnDest,
    MissingNodeId,
}

impl From<IError> for ScophError {
    fn from(value: IError) -> Self {
        match value {
            IError::MissingChild { .. } => Self::MissingChild,
            IError::MissingLayerOnDest(_) => Self::MissingLayerOnDest,
            IError::MissingLayerOnSrc(_) => Self::MissingLayerOnSrc,
            IError::MissingNodeId(_) => Self::MissingNodeId,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_strerror(error: ScophError) -> *const c_char {
    match error {
        ScophError::LoadFailed => c"Failed to load file",
        ScophError::Ok => c"No error",
        ScophError::MissingChild => c"Missing child of node",
        ScophError::MissingLayerOnSrc => c"Missing layer on source",
        ScophError::MissingLayerOnDest => c"Missing layer on destination",
        ScophError::MissingNodeId => c"Missing a node with the given ID",
    }
    .as_ptr()
}
