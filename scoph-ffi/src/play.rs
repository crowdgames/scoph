use std::ffi::{c_char, c_uint};
///cbindgen:ignore
use scoph::common::{PlayerChoices, TRRBTPattern};
use crate::{ScophIntrp, error::ScophError, pattern::Pattern};

/// Represents the set of choices that the interpreter is waiting for the player to pick
pub struct ScophPlayerChoices(*const c_char, *const PlayerChoices);

/// Get the count of possible rewrites that the player can choose
#[unsafe(no_mangle)]
pub extern "C" fn scoph_choices_count(choices: ScophPlayerChoices) -> c_uint {
    unsafe { (*choices.1).len() as c_uint }
}

/// Get the count of matches of the left-hand side of the rewrite at the given index.
#[unsafe(no_mangle)]
pub extern "C" fn scoph_choices_match_count(
    choices: ScophPlayerChoices,
    rewrite_index: c_uint,
) -> c_uint {
    unsafe { (*choices.1).get_index(rewrite_index as usize) }
        .map(|(_, matches)| matches.len() as c_uint)
        .unwrap_or(0)
}

/// Accesses the lhs and rhs of a rewrite in a set of player choices. Will panic if the index is
/// invalid. See [scoph_choices_count].
#[unsafe(no_mangle)]
pub extern "C" fn scoph_choices_rewrites(
    choices: ScophPlayerChoices,
    rewrite_index: c_uint,
    lhs: *mut Pattern,
    rhs: *mut Pattern,
) {
    let (lrw, rrw) = unsafe { &*choices.1 }
        .get_index(rewrite_index as usize)
        .map(|(rewrite, _)| rewrite)
        .expect("Non-existent index for choice");

    unsafe {
        *lhs = Pattern(lrw as *const TRRBTPattern);
        *rhs = Pattern(rrw as *const TRRBTPattern);
    }
}

/// Get the coordinates at the specified index of the specified rewrite
#[unsafe(no_mangle)]
pub extern "C" fn scoph_choices_coords(
    choices: ScophPlayerChoices,
    rewrite_index: c_uint,
    match_index: c_uint,
    x: *mut c_uint,
    y: *mut c_uint,
) {
    let (rwx, rwy) = unsafe { &*choices.1 }
        .get_index(rewrite_index as usize)
        .map(|(_, coords)| coords[match_index as usize])
        .expect("Non-existent index for choice");

    unsafe {
        *x = rwx as c_uint;
        *y = rwy as c_uint;
    }
}

/// Get the ID of the player associated with these choices
#[unsafe(no_mangle)]
pub extern "C" fn scoph_choices_player_id(choices: ScophPlayerChoices) -> *const c_char {
    choices.0
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_player_choose(
    intrp: *mut ScophIntrp,
    rewrite_index: c_uint,
    match_index: c_uint,
) -> ScophError {
    let intrp = unsafe { &mut (*intrp).0 };

    match intrp.player_choose(rewrite_index as usize, match_index as usize) {
        Ok(_) => ScophError::Ok,
        Err(e) => e.into(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn scoph_intrp_check_choices(
    intrp: *const ScophIntrp,
    choices: *mut ScophPlayerChoices,
) -> ScophError {
    let intrp = unsafe { &(*intrp).0 };

    struct UnsafeVisitor(*mut ScophPlayerChoices);

    impl scoph::interpreter::Visitor for UnsafeVisitor {
        fn present_choices(
            &mut self,
            player_id: &scoph::common::PlayerId,
            choices: &PlayerChoices,
        ) {
            unsafe {
                (*self.0) =
                    ScophPlayerChoices(player_id.0.as_ptr(), choices as *const PlayerChoices)
            }
        }
    }

    if intrp.check_choices(&mut UnsafeVisitor(choices)) {
        ScophError::Ok
    } else {
        ScophError::NoChoices
    }
}
