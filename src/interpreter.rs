mod err;
mod tree;

pub use err::IError;

use petgraph::graph::NodeIndex;
pub use tree::BehaviorTree;

use crate::common::TRRBTPattern;

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

enum Eval {
    Finished(bool),
    Push(NodeIndex),
}

impl Call {
    fn evaluate(&mut self, tree: &BehaviorTree, board: &mut TRRBTPattern) -> Result<Eval, IError> {
        use crate::common::NodeAction as N;
        match tree.get_node_action(self.node_index) {
            N::LoopUntilAll => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                if self.child as usize >= neighbors.len() {
                    if let StackState::LoopUntilAll {
                        any_success,
                        all_failed: true,
                    } = &self.state
                    {
                        return Ok(Eval::Finished(*any_success));
                    } else {
                        self.child = 0;
                    }
                }
                Ok(Eval::Push(neighbors[self.child as usize]))
            }
            N::SetBoard { pattern } => {
                *board = (*pattern).clone();
                Ok(Eval::Finished(true))
            }
            N::Rewrite { lhs, rhs } => {
                use rand::seq::IteratorRandom;
                let mut layer_replacements = vec![];
                for (layer_name, coords) in board.matches(lhs) {
                    // for now does a random rewrite at a valid position
                    // TODO: account for player choices
                    if let Some(coords) = coords.choose(&mut rand::rng()) {
                        layer_replacements.push((String::from(layer_name), coords));
                    }
                }

                let result = Eval::Finished(!layer_replacements.is_empty());
                for (layer_name, (x, y)) in layer_replacements {
                    board.rewrite_at(x, y, &layer_name, rhs)?;
                }
                Ok(result)
            }
            _ => todo!(),
        }
    }

    fn child_finished(&mut self, succeeded: bool) {
        use StackState as S;
        match self.state {
            S::LoopUntilAll {
                ref mut any_success,
                ref mut all_failed,
            } => {
                if succeeded {
                    *any_success = true;
                    *all_failed = false;
                }
            }
            _ => {}
        }
    }
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
        use crate::common::NodeAction::*;
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

    pub fn startup<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.startup(self.tree.name(), self.tree.description());
    }

    pub fn step<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), IError> {
        // peek at the top of the call stack, perform node actions to the tree/board. Provide a
        // call result indicating if the node succeeded.
        let call_result = if let Some(call) = self.call_stack.last_mut() {
            call.evaluate(&self.tree, &mut self.board)?
        } else {
            return Ok(()); // stack is empty, interpreter is finished
        };

        match call_result {
            Eval::Finished(succeeded) => {
                self.call_stack.pop();
                if let Some(call) = self.call_stack.last_mut() {
                    call.child_finished(succeeded);
                }
            }
            Eval::Push(node_index) => {
                self.push_call(node_index);
            }
        }

        visitor.view_board(&self.board);
        Ok(())
    }

    pub fn run<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), IError> {
        self.startup(visitor);
        while !self.call_stack.is_empty() {
            self.step(visitor)?;
        }
        Ok(())
    }
}
