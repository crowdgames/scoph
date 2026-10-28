use std::ffi::{CStr, OsStr, c_char};

/// cbindgen:ignore
use scoph::interpreter::{BehaviorTree, Interpreter, Status, Outcome};

pub struct ScophIntrp(Interpreter);

mod board;
mod error;

use error::ScophError;

/// Reads a TRRBT json file and constructs an interpreter. The caller is responsible for freeing the
/// interpreter from memory with [`scoph_intrp_free`]
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

/// Frees a scoph interpreter from memory.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_free(intrp: *mut *mut ScophIntrp) {
    unsafe { drop(Box::from_raw(*intrp)) }
}

/// The current status of a scoph interpreter
#[repr(C)]
#[derive(Default)]
pub enum ScophIntrpResult {
    #[default]
    /// The default state of an interpreter, means the program hasn't started running yet
    Unstarted,
    /// The interpreter is still running and hasn't finished
    InProgress,
    /// The interpreter encountered an error and can no longer run
    Failed,
    /// The interpreter is finished running the game. Either by completing running the tree or
    /// finding a winner.
    Finished,
}

/// Makes an interpreter take a step in its execution. [res] is a [ScophIntrpResult] that signifies
/// the status of the interpreter after the step.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_step(
    intrp: *mut ScophIntrp,
    res: *mut ScophIntrpResult,
) -> ScophError {
    let intrp = unsafe { &mut (*intrp).0 };

    match intrp.step() {
        Ok(finished) => unsafe {
            *res = match finished {
                Status::Finished => ScophIntrpResult::Finished,
                Status::InProgress => ScophIntrpResult::InProgress,
                Status::Unstarted => ScophIntrpResult::Unstarted,
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

#[repr(C)]
pub enum ScophOutcomeType {
    Win,
    Lose,
    Tie,
}

#[repr(C)]
pub struct ScophOutcome {
    outcome: ScophOutcomeType,

    /// Id of the player related to the outcome. Will be "nobody" if it's a tie
    player_id: *const c_char,
}

/// If an interpreter returns its finished then users can check the outcome
#[unsafe(no_mangle)]
pub extern "C" fn scoph_get_outcome(
    intrp: *const ScophIntrp,
    outcome: *mut ScophOutcome,
) -> ScophError {
    let intrp = unsafe { &(*intrp).0 };

    match intrp.outcome() {
        Some(Outcome::Win(pid)) => unsafe {
            (*outcome).outcome = ScophOutcomeType::Win;
            (*outcome).player_id = pid.0.as_ptr();
        },
        Some(Outcome::Lose(pid)) => unsafe {
            (*outcome).outcome = ScophOutcomeType::Lose;
            (*outcome).player_id = pid.0.as_ptr();
        },
        Some(Outcome::Tie) => unsafe {
            (*outcome).outcome = ScophOutcomeType::Tie;
            (*outcome).player_id = c"nobody".as_ptr();
        },
        None => return ScophError::NoOutcome,
    }

    ScophError::Ok
}
