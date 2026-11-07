use std::{
    assert_matches,
    ffi::{CStr, CString},
    str::FromStr,
};

use super::*;
use crate::{
    common::toodee_pattern,
    interpreter::{
        transforms::Transform,
        visitor::{Bare, Dummy, Printer, StackTraceOnly},
    },
};
use anyhow::Result;
use pretty_assertions::assert_eq;
use serde_json::json;

pub mod loops;
pub mod player_choices;
pub mod transforms;

struct Comparer {
    expected_name: String,
    expected_description: String,
    board_states: Vec<TRRBTPattern>,
    current_stack: (Vec<Frame>, Vec<Transform>, Vec<Frame>),
}

impl Visitor for Comparer {
    fn startup(&mut self, tree_name: &CStr, tree_description: &CStr) {
        assert_eq!(CString::from_str(&self.expected_name).unwrap(), tree_name);
        assert_eq!(
            CString::from_str(&self.expected_description).unwrap(),
            tree_description
        );
    }

    fn view_board(&mut self, board: &TRRBTPattern) {
        self.board_states.push(board.clone());
    }

    fn stack_trace(
        &mut self,
        call_stack: &[Frame],
        transform_stack: &[Transform],
        passed_frames: &[Frame],
    ) {
        self.current_stack = (
            call_stack.to_vec(),
            transform_stack.to_owned(),
            passed_frames.to_vec(),
        );
    }
}

// player choice tests
struct Chooser {
    cmp: Comparer,
    expected_choices: PlayerChoices,
}

impl Visitor for Chooser {
    fn startup(&mut self, tree_name: &CStr, tree_description: &CStr) {
        self.cmp.startup(tree_name, tree_description);
    }

    fn view_board(&mut self, board: &TRRBTPattern) {
        self.cmp.view_board(board);
    }

    fn present_choices(&mut self, player_id: &PlayerId, choices: &PlayerChoices) {
        assert_eq!(
            choices, &self.expected_choices,
            "Didn't get expected choices for {:?}\n{:#?}",
            player_id, self.cmp.current_stack
        );
    }
}
