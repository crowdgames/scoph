//! # Scoph 🐠
//!
//! `scoph` is an embeddable interpreter for the [TRRBT programming language](https://osf.io/btqcj/files/j6y7p). With TRRBT, a user can implement games like [Tic-Tac-Toe](https://crowdgames.github.io/trrbt/edengine.html#tic-tac-toe), [Minesweeper](https://crowdgames.github.io/trrbt/edengine.html#sweeper), [Sokoban](https://crowdgames.github.io/trrbt/edengine.html#soko-demo),
//! and etc. in a simple-to-understand graph format. Scoph acts as a TRRBT interpreter that can run these games agnostically.
//! Developers can include scoph in their project as a rust dependency, or as a static C library.
//! Then they can hook their gamedev tool of choice to scoph. This eliminates the need to
//! reimplement these games for each development platform. If a game is valid TRRBT code, scoph will
//! execute it regardless of game engine or framework!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod parse;

/// Common types in TRRBT
pub mod common;
/// TRRBT interpreter types
pub mod interpreter;

/// The bare neccessities to use scoph in a rust project
pub mod prelude {
    use super::*;
    pub use alloc::{
        borrow::ToOwned,
        boxed::Box,
        collections::{BTreeMap, BTreeSet},
        ffi::CString,
        str::FromStr,
        format,
        rc::Rc,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    pub use common::{BehaviorTree, NodeIndex, TRRBTPattern};
    pub use interpreter::Interpreter;
}

