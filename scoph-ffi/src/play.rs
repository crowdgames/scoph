use std::ffi::c_char;

use scoph::common::{PlayerChoices, PlayerId};

use crate::{ScophIntrp, error::ScophError};

pub struct ScophPlayerChoices(*const c_char, *const PlayerChoices);

#[unsafe(no_mangle)]
pub extern "C" fn scoph_get_choices(
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
