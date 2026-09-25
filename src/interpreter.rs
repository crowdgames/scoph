mod tree;

use petgraph::graph::NodeIndex;
pub use tree::BehaviorTree;

use crate::common::{NodeAction, TRRBTPattern};

#[cfg(test)]
mod test;

enum StackState {
    LoopUntilAll { any_success: bool, all_failed: bool },
    SingleAction,
}

struct Call {
    node_index: NodeIndex,
    child: u32,
    state: StackState,
}

/// Allows users to view into an interpreter's current state as it runs
pub trait Visitor {
    /// Provides the name and description of the tree to be displayed however deemed fit.
    fn startup(&mut self, tree_name: &str, tree_description: &str) {
        println!("= Name: ===============");
        println!("{tree_name}");
        println!("= Description: =========");
        println!("{tree_description}");
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
    fn stack_trace(&mut self, tree: &BehaviorTree, call_stack: &[Call]) {
        for (frame_index, call) in call_stack.iter().enumerate() {
            println!("= {frame_index} ====================");
            println!("| {:?}", tree.get_node_action(call.node_index));

            let child_count = tree.get_node_neighbors(call.node_index).count();
            if child_count > 0 {
                println!("| child index: {} out of {}", call.child, child_count);
            }
        }
    }
}

/// Default visitor that prints to stdout
pub struct Printer;
impl Visitor for Printer {}

pub struct Interpreter {
    tree: BehaviorTree,
    board: TRRBTPattern,
    call_stack: Vec<Call>,
}

impl Interpreter {
    pub fn new(tree: BehaviorTree) -> Self {
        let root = tree.root();
        let mut out = Self {
            tree,
            board: Default::default(),
            call_stack: Default::default(),
        };

        out.push_call(root);

        out
    }

    fn push_call(&mut self, node_index: NodeIndex) {
        use NodeAction::*;
        let state = match self.tree.get_node_action(node_index) {
            LoopUntilAll => StackState::LoopUntilAll {
                any_success: false,
                all_failed: true,
            },
            SetBoard { .. } | Rewrite { .. } => StackState::SingleAction,
            _ => todo!(),
        };

        self.call_stack.push(Call {
            node_index,
            child: 0,
            state,
        });
    }

    pub fn run<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.startup(self.tree.name(), self.tree.description());
        while !self.call_stack.is_empty() {
            let mut call_result = None;

            // peek at the top of the call stack, perform node actions to the tree/board. Provide a
            // call result indicating if the node succeeded.
            if let Some(call) = self.call_stack.iter_mut().last() {
                use NodeAction::*;
                match self.tree.get_node_action(call.node_index) {
                    LoopUntilAll => {
                        let neighbors: Vec<_> =
                            self.tree.get_node_neighbors(call.node_index).collect();
                        if call.child >= neighbors.len() as u32 {
                            // split this into a function so I can return instead
                            if let StackState::LoopUntilAll {
                                any_success,
                                all_failed: true,
                            } = &call.state
                            {
                                call_result = Some(*any_success);
                            } else {
                                call.child = 0;
                            }
                        }

                        if call_result.is_none() {
                            let push_node_index = neighbors.get(call.child as usize);
                            if let Some(&push_node_index) = push_node_index {
                                self.push_call(push_node_index);
                            } else {
                                // TODO: make an exception/error raising system

                                // error, their is no neighbor at the expected child index
                            }
                        }
                    }
                    SetBoard { pattern } => {
                        self.board = (*pattern).clone();
                        call_result = Some(true);
                    }
                    Rewrite { lhs, rhs } => todo!(),
                    _ => todo!("Not implemented yet"),
                }
            }

            if let Some(did_succeed) = call_result {
                self.call_stack.pop();

                if let Some(call) = self.call_stack.iter_mut().last() {
                    match call.state {
                        StackState::LoopUntilAll {
                            ref mut any_success,
                            ref mut all_failed,
                        } => {
                            if did_succeed {
                                *any_success = true;
                                *all_failed = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            visitor.view_board(&self.board);
        }
    }
}
