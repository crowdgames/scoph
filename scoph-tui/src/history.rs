use std::ffi::CStr;

use scoph::common::TRRBTPattern;
use scoph::interpreter::StackFrameInfo;
use scoph::prelude::BehaviorTree;

use scoph::interpreter::Visitor;

pub(crate) struct ReplHistory {}

impl ReplHistory {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Visitor for ReplHistory {
    fn startup(&mut self, _tree_name: &CStr, _tree_description: &CStr) {
        todo!()
    }

    fn view_board(&mut self, _board: &TRRBTPattern) {
        todo!()
    }

    fn stack_trace(
        &mut self,
        _tree: &BehaviorTree,
        _call_stack: impl Iterator<Item = StackFrameInfo>,
    ) {
        todo!()
    }
}
