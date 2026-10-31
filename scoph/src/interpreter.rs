mod err;
mod stack;
mod transforms;
mod tree;
mod visitor;

use std::rc::Rc;

pub use err::IError;
use stack::ExecutionFrame;
pub use stack::Outcome;
pub use visitor::StackFrameInfo;
pub use visitor::Visitor;

use petgraph::graph::NodeIndex;
pub use tree::BehaviorTree;

use transforms::TransformStack;

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
    tree: Rc<BehaviorTree>,
    board: TRRBTPattern,
    started: bool,
    waiting_for_choice: bool,
    execution_stack: Vec<ExecutionFrame>,
    passed_frames: Vec<ExecutionFrame>,
    transform_stack: TransformStack,
    outcome: Option<Outcome>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Status {
    #[default]
    Unstarted,
    InProgress,
    Finished,
}

impl Interpreter {
    /// Construct a new interpreter that will run on the provided [BehaviorTree]
    pub fn new(tree: BehaviorTree) -> Self {
        let root = tree.root();
        let mut out = Self {
            tree: Rc::new(tree),
            board: Default::default(),
            execution_stack: Default::default(),
            passed_frames: Default::default(),
            started: Default::default(),
            waiting_for_choice: Default::default(),
            transform_stack: Default::default(),
            outcome: None,
        };

        out.push_frame(root);

        out
    }

    fn push_frame(&mut self, node_index: NodeIndex) {
        let frame = ExecutionFrame::new(self.tree.clone(), node_index);
        self.execution_stack.push(frame);
    }

    /// Returns a reference to this interpreter's behavior tree
    pub fn tree(&self) -> &BehaviorTree {
        &self.tree
    }

    /// Invokes `Visitor::startup`. Can only be called once, intended to call init functions of game
    /// engines using scoph that don't use any of the [`Interpreter::run`] methods
    pub fn startup<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), IError> {
        if !self.started {
            visitor.startup(self.tree.name(), self.tree.description());
            self.started = true;
            Ok(())
        } else {
            Err(IError::AlreadyStarted)
        }
    }

    /// Invokes `Visitor::view_board`
    pub fn view_board<V: Visitor>(&self, visitor: &mut V) {
        visitor.view_board(&self.board);
    }

    fn restack_passed_frames(&mut self) {
        assert!(
            !self.passed_frames.is_empty(),
            "Can't restack frames if their are none"
        );
        while let Some(passer) = self.passed_frames.pop() {
            self.execution_stack.push(passer);
        }
    }

    /// Causes the interpreter to take a step in its evaluation. If successful, will return an
    /// option that if some means the game is concluded with the provided outcome. Otherwise gives an
    /// interpreter error
    pub fn step(&mut self) -> Result<Status, IError> {
        if !self.started {
            return Err(IError::HasntStarted);
        }

        if self.outcome.is_some() {
            return Ok(Status::Finished);
        }

        // peek at the top of the call stack, perform node actions to the tree/board. Provide a
        // call result indicating if the node succeeded.
        let call_result = if let Some(call) = self.execution_stack.last_mut() {
            call.evaluate(&mut self.board, &mut self.transform_stack)?
        } else {
            // stack is empty
            if self.outcome.is_none() {
                self.outcome = Some(Outcome::Tie);
            }
            return Ok(Status::Finished);
        };

        match call_result {
            Eval::Finished(succeeded) => {
                // pop frame that finished
                let _prev = self.execution_stack.pop();

                // if our passed_frames is full, that means we need to kill that section of the tree
                // because we just had a success
                if !self.passed_frames.is_empty() {
                    self.passed_frames.clear();
                }

                // collect transform frames
                let untransformed = self.execution_stack.last().is_some_and(|frame| !frame.action().is_transform());
                let mut passing_frames = vec![];
                while let Some(passer) = self
                    .execution_stack
                    .pop_if(|frame| frame.child_finished(succeeded, untransformed).is_some())
                {
                    passing_frames.push(passer);
                }
                self.passed_frames.append(&mut passing_frames);
            }

            Eval::Push(node_index) => {
                if self
                    .passed_frames
                    .last()
                    .is_some_and(|frame| frame.source_index() == node_index)
                {
                    self.restack_passed_frames();
                } else {
                    self.push_frame(node_index);
                }
            }

            Eval::TransformFinished => {
                assert!(self.passed_frames.is_empty());
                self.execution_stack.pop();
            }

            Eval::TransformPush(call) => {
                assert!(self.passed_frames.is_empty());
                self.execution_stack.push(call);
            }

            Eval::End(outcome) => {
                self.execution_stack.clear();
                self.passed_frames.clear();
                self.outcome = Some(outcome);
                return Ok(Status::Finished);
            }
        }

        Ok(Status::InProgress)
    }

    /// Runs an entire scoph program without stopping. Will run the visitor methods on the
    /// specified capabilities.
    pub fn run<V: Visitor, C: Capabilities>(&mut self, visitor: &mut V) -> Result<(), IError> {
        self.startup(visitor)?;

        loop {
            match self.step() {
                Ok(status) => {
                    if status == Status::Finished {
                        break;
                    }

                    if C::can_view_board() {
                        visitor.view_board(&self.board);
                    }
                }
                Err(intrp_err) => {
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

    /// Check if the game is over and has an outcome
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }
}
