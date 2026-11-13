use core::ffi::c_char;

use scoph::interpreter::IError;

/// Represents an error may encounter when running TRRBT code.
#[repr(C)]
#[allow(nonstandard_style)]
pub enum ScophError {
    /// No error, can continue
    Ok,
    /// Json file was not parseable for interpreter
    BadJson,
    /// Failed to load a provided json file
    LoadFailed,
    /// A node in the tree is missing a child at a specific index
    MissingChild,
    /// Attempt to access a layer on a pattern that doesn't exist.
    MissingLayerOnSrc,
    /// Attempt to write a pattern to a layer on a pattern that doesn't exists.
    MissingLayerOnDest,
    /// Attempt to reference a node with an id that doesn't exist in the tree.
    MissingNodeId,
    /// Tried getting checking the outcome of an interpreter while it's unfinished.
    NoOutcome,
    /// Tried calling startup again after the interpreter was already initialized
    AlreadyStarted,
    /// Tried calling step before the interpreter was initialized with startup
    HasntStarted,
    /// Tree contains an x-link node to a target id that doesn't exist in the tree
    BadLink,
    /// A player node in the tree has a child that is neither transform nor rewrite
    BadPlayerChild,
    /// Tried to make a choice when the interpreter doesn't have any
    NoChoices,
    /// Tried to make a choice at a coord where a match doesn't exist
    BadChoice,
}

impl From<IError> for ScophError {
    fn from(value: IError) -> Self {
        match value {
            IError::MissingChild { .. } => Self::MissingChild,
            IError::MissingLayerOnDest(_) => Self::MissingLayerOnDest,
            IError::MissingLayerOnSrc(_) => Self::MissingLayerOnSrc,
            IError::MissingNodeId(_) => Self::MissingNodeId,
            IError::AlreadyStarted => Self::AlreadyStarted,
            IError::HasntStarted => Self::HasntStarted,
            IError::BadLink { .. } => Self::BadLink,
            IError::BadPlayerChild(_, _) => Self::BadPlayerChild,
            IError::NoChoices => Self::NoChoices,
            IError::BadChoice => Self::BadChoice,
        }
    }
}

// TODO: this some terrible error handling. There has got to be a better way than this, that
// actually tells users where in the tree the error occurred.

/// Gives a more detailed string of the error code that can be printed
#[unsafe(no_mangle)]
pub extern "C" fn scoph_strerror(error: ScophError) -> *const c_char {
    match error {
        ScophError::LoadFailed => c"Failed to load file",
        ScophError::Ok => c"No error",
        ScophError::BadJson => c"Json file was not a valid UTF-8 string",
        ScophError::MissingChild => c"Missing child of node",
        ScophError::MissingLayerOnSrc => c"Missing layer on source",
        ScophError::MissingLayerOnDest => c"Missing layer on destination",
        ScophError::MissingNodeId => c"Missing a node with the given ID",
        ScophError::NoOutcome => c"Interpreter hasn't finished",
        ScophError::AlreadyStarted => {
            c"Tried calling startup on an interpreter that already started"
        }
        ScophError::HasntStarted => {
            c"Tried calling step on an interpreter that hansn't called startup yet"
        }
        ScophError::BadLink => c"Linked to a target that doesn't exist in the tree",
        ScophError::BadPlayerChild => {
            c"A player node has a child that is neither transform nor rewrite"
        }
        ScophError::NoChoices => c"Tried to make a choice when the interpreter doesn't have any",
        ScophError::BadChoice => c"Tried to make a choice at a coord where a match doesn't exist",
    }
    .as_ptr()
}
