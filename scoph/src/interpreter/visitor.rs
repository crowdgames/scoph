use std::ffi::CStr;

use petgraph::graph::NodeIndex;

use crate::{
    common::{PlayerId, TRRBTPattern},
    interpreter::{BehaviorTree, stack::PlayerChoices},
};

// tells the interpreter what the visitor is allowed to do
pub trait Capabilities {
    fn can_view_board() -> bool {
        false
    }
    fn can_stack_trace() -> bool {
        false
    }
}

/// Represents that the interpreter doesn't need to do anything with the visitor.
#[derive(Default)]
pub struct Bare;
impl Capabilities for Bare {}

/// For visitors that only want to view the board and nothing else.
#[derive(Default)]
pub struct ViewBoardOnly;
impl Capabilities for ViewBoardOnly {
    fn can_view_board() -> bool {
        true
    }
}

/// Does everything that a visitor can do during the lifecycle of the interpreter. This is the
/// default choice for when an interpreter steps or runs.
#[derive(Default)]
pub struct Full;
impl Capabilities for Full {
    fn can_view_board() -> bool {
        true
    }

    fn can_stack_trace() -> bool {
        true
    }
}

pub type StackFrameInfo = (NodeIndex, u32);

/// Allows users to view into an interpreter's current state as it runs
pub trait Visitor {
    /// Provides the name and description of the tree to be displayed however deemed fit.
    fn startup(&mut self, tree_name: &CStr, tree_description: &CStr) {
        println!("= Name: ===============");
        println!("{tree_name:?}");
        println!("= Description: =========");
        println!("{tree_description:?}");
    }

    /// Provides a look into the current state of the tile map.
    fn view_board(&mut self, board: &TRRBTPattern) {
        for (layer_name, layer_rows) in board.layers() {
            println!("= {layer_name} ===========");
            for row in layer_rows {
                for cell in row {
                    print!("{} ", cell);
                }
                println!("");
            }
        }
    }

    /// Prints out the current call stack. By default prints to stdout.
    fn stack_trace(
        &mut self,
        tree: &BehaviorTree,
        call_stack: impl Iterator<Item = StackFrameInfo>,
    ) {
        for (frame_index, (node_index, child_index)) in call_stack.enumerate() {
            println!("= {frame_index} ====================");
            println!("| {:?}", tree.get_node_action(node_index));

            let child_count = tree.get_node_neighbors(node_index).count();
            if child_count > 0 {
                println!("| child index: {} out of {}", child_index, child_count);
            }
        }
        println!("<===========================>");
    }

    fn present_choices(&mut self, player_id: &PlayerId, choices: &PlayerChoices) {
        println!("PID:({:?}) {:?}", player_id, choices);
    }
}

/// Default visitor that prints to stdout
pub struct Printer;
impl Visitor for Printer {}

// Dummy visitor that doesn't do anything
pub struct Dummy;
impl Visitor for Dummy {
    fn startup(&mut self, _tree_name: &CStr, _tree_description: &CStr) {}

    fn view_board(&mut self, _board: &TRRBTPattern) {}

    fn stack_trace(
        &mut self,
        _tree: &BehaviorTree,
        _call_stack: impl Iterator<Item = StackFrameInfo>,
    ) {
    }

    fn present_choices(&mut self, _player_id: &PlayerId, _choices: &PlayerChoices) {}
}
