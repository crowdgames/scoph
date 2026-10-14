use scoph::common::TRRBTPattern;
use scoph::interpreter::BehaviorTree;
use scoph::interpreter::StackFrameInfo;

use scoph::interpreter::Visitor;

pub(crate) struct ReplHistory {

}

impl ReplHistory {
    pub(crate) fn new() -> Self {
        Self {  }
    }
}

impl Visitor for ReplHistory {
    fn startup(&mut self, tree_name: &str, tree_description: &str) {
    }

    fn view_board(&mut self, board: &TRRBTPattern) {
    }

    fn stack_trace(
        &mut self,
        tree: &BehaviorTree,
        call_stack: impl Iterator<Item = StackFrameInfo>,
    ) {
    }
}

