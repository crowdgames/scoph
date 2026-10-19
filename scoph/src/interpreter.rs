mod err;
mod stack;
mod tree;
mod visitor;

use std::collections::HashMap;

pub use err::IError;
use stack::Call;
pub use visitor::StackFrameInfo;
pub use visitor::Visitor;

use petgraph::graph::NodeIndex;
pub use tree::BehaviorTree;

use crate::{
    common::TRRBTPattern,
    interpreter::{
        stack::Eval,
        visitor::{Capabilities, Full},
    },
};

#[cfg(test)]
mod test;

/// The core of running a scoph game. Holds onto a behavior tree and runs until the game finishes.
pub struct Interpreter {
    tree: BehaviorTree,
    board: TRRBTPattern,
    breakpoints: HashMap<NodeIndex, String>,
    call_stack: Vec<Call>,
}

impl Interpreter {
    pub fn new(tree: BehaviorTree) -> Self {
        let root = tree.root();
        let mut out = Self {
            tree,
            board: Default::default(),
            breakpoints: HashMap::new(),
            call_stack: Default::default(),
        };

        out.push_call(root);

        out
    }

    pub fn add_breakpoint(&mut self, id: &str) -> Result<(), IError> {
        if let Some(index) = self.tree.get_node_by_id(id) {
            self.breakpoints.insert(index, id.to_string());
            Ok(())
        } else {
            Err(IError::MissingNodeId(id.to_string()))
        }
    }

    pub fn remove_breakpoint(&mut self, id: &str) -> Result<(), IError> {
        if let Some(index) = self.tree.get_node_by_id(id) {
            self.breakpoints.remove(&index);
            Ok(())
        } else {
            Err(IError::MissingNodeId(id.to_string()))
        }
    }

    pub fn at_breakpoint(&self) -> bool {
        self.call_stack
            .last()
            .is_some_and(|c| self.breakpoints.contains_key(&c.info().0))
    }

    fn push_call(&mut self, node_index: NodeIndex) {
        self.call_stack.push(Call::new(&self.tree, node_index));
    }

    /// Returns a reference to this interpreter's behavior tree
    pub fn tree(&self) -> &BehaviorTree {
        &self.tree
    }

    /// Invokes `Visitor::startup`
    pub fn startup<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.startup(self.tree.name(), self.tree.description());
    }

    /// Invokes `Visitor::view_board`
    pub fn view_board<V: Visitor>(&self, visitor: &mut V) {
        visitor.view_board(&self.board);
    }

    /// Causes the interpreter to take a step in its evaluation. If successful, will return a
    /// boolean that indicates the TRRBT program is finished with `true`. Otherwise gives an
    /// interpreter error
    pub fn step(&mut self) -> Result<bool, IError> {
        // peek at the top of the call stack, perform node actions to the tree/board. Provide a
        // call result indicating if the node succeeded.
        let call_result = if let Some(call) = self.call_stack.last_mut() {
            call.evaluate(&self.tree, &mut self.board)?
        } else {
            return Ok(true); // stack is empty, interpreter is finished
        };

        match call_result {
            Eval::Finished(succeeded) => {
                let _prev = self.call_stack.pop();
                if let Some(call) = self.call_stack.last_mut() {
                    call.child_finished(succeeded);
                }
            }
            Eval::Push(node_index) => {
                self.push_call(node_index);
            }
        }

        Ok(false)
    }

    pub fn run<V: Visitor, C: Capabilities>(&mut self, visitor: &mut V) -> Result<(), IError> {
        if C::can_startup() {
            visitor.startup(self.tree.name(), self.tree.description());
        }

        loop {
            match self.step() {
                Ok(finished) => {
                    if finished {
                        break;
                    }

                    if C::can_view_board() {
                        visitor.view_board(&self.board);
                    }
                }
                Err(intrp_err) => {
                    if C::can_stack_trace() {
                        visitor.stack_trace(&self.tree, self.call_stack.iter().map(Call::info));
                    }

                    return Err(intrp_err);
                }
            }
        }
        Ok(())
    }

    /// Run an interpreter the whole way through with all capabilities
    pub fn run_full<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), IError> {
        self.run::<V, Full>(visitor)
    }

    /// get the size of a specified layer if it exists
    pub fn board_layer_size(&self, layer_name: &str) -> Option<(usize, usize)> {
        self.board
            .pattern_sizes()
            .find_map(|(name, (width, height))| (name == layer_name).then_some((width, height)))
    }
}
