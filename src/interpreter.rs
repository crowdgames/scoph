mod tree;

use petgraph::graph::NodeIndex;
use toodee::TooDee;
use tree::BehaviorTree;

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

enum Evaluation {
    Finished(bool),
    Push(NodeIndex),
}

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

    pub fn run(&mut self) {
        while !self.call_stack.is_empty() {
            let mut call_result = None;
            if let Some(call) = self.call_stack.iter_mut().last() {
                use NodeAction::*;
                match self.tree.get_node_action(call.node_index) {
                    LoopUntilAll => todo!(),
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
                            any_success,
                            all_failed,
                        } => todo!(),
                        _ => {}
                    }
                }
            }
        }
    }
}
