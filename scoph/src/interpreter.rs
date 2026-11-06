mod err;
mod stack;
mod transforms;
mod visitor;

use std::rc::Rc;

pub use err::IError;
use stack::ExecutionFrame;
pub use stack::Outcome;
pub use visitor::StackFrameInfo;
pub use visitor::Visitor;

use crate::common::BehaviorTree;
use petgraph::graph::NodeIndex;

use transforms::TransformStack;

use crate::common::PlayerId;
use crate::interpreter::stack::PlayerChoices;
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
    player_choices: Option<(PlayerId, PlayerChoices)>,
    execution_stack: Vec<ExecutionFrame>,
    passed_frames: Vec<ExecutionFrame>,
    transform_stack: TransformStack,
    outcome: Option<Outcome>,
}

/// The status of an interpreter after a step.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Status {
    #[default]
    /// The interpreter has not had startup called.
    Unstarted,
    /// The game is still running.
    InProgress,
    /// The game has concluded with an [Outcome].
    Finished,
    /// The game is waiting for the player to make a choice.
    WaitingForChoice,
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
            player_choices: Default::default(),
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

    /// Will attempt to call [Visitor::present_choices] on the provided visitor. Returns true if the
    /// choices exist and were presented, returns false if the interpreter doesn't have any choices
    /// to make.
    ///
    /// It's reccomended to use this method as a way to get your engine to collect choices so the
    /// choice presentation is up to the game developer.
    pub fn check_choices<V: Visitor>(&self, visitor: &mut V) -> bool {
        if let Some((pid, matches)) = &self.player_choices {
            visitor.present_choices(pid, matches);
            true
        } else {
            false
        }
    }

    /// Instructs the interpreter of the player's choice for a rewrite. Will fail if the
    /// interpreter's latest status wasn't [Status::WaitingForChoice]
    pub fn player_choose(
        &mut self,
        rewrite_index: usize,
        match_index: usize,
    ) -> Result<(), IError> {
        let (_, choices) = self.player_choices.as_ref().ok_or(IError::NoChoices)?;

        let (rhs, (x, y)) = choices
            .get_index(rewrite_index)
            .map(|((_, rhs), coords)| (rhs, coords[match_index]))
            .ok_or_else(|| IError::BadChoice)?;

        self.board.rewrite_at(x, y, rhs)?;

        // player succeeded, pass the result to its parent in the stack
        let _ = self.handle_eval(Eval::Finished(true))?;

        self.player_choices = None;

        Ok(())
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
        } else if self.outcome.is_some() {
            return Ok(Status::Finished);
        } else if self.player_choices.is_some() {
            return Ok(Status::WaitingForChoice);
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

        self.handle_eval(call_result)
    }

    fn handle_eval(&mut self, call_result: Eval) -> Result<Status, IError> {
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
                let untransformed = self
                    .execution_stack
                    .last()
                    .is_some_and(|frame| !frame.action().is_transform());
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
                if let Some(frame) = self.execution_stack.last_mut()
                    // && !frame.action().is_transform()
                {
                    if frame.action().is_transform() {
                        self.transform_stack.pop();
                    } else {
                        frame.next_branch();
                    }
                }
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

            Eval::PlayerChoices { pid, matches } => {
                self.player_choices = Some((pid, matches));
                return Ok(Status::WaitingForChoice);
            }
        }

        Ok(Status::InProgress)
    }

    /// Runs a TRRBT program without calling startup. Will break if the interpreter is waiting for a
    /// player choice.
    ///
    /// NOTE: will fail if startup is not called prior. It's reccomended to use [Interpreter::run]
    /// to start your program.
    pub fn resume<V: Visitor, C: Capabilities>(&mut self, visitor: &mut V) -> Result<(), IError> {
        loop {
            let status = self.step()?;
            match status {
                Status::Finished => break,
                Status::WaitingForChoice => {
                    let Some((pid, matches)) = self.player_choices.as_ref() else {
                        unreachable!();
                    };
                    visitor.present_choices(pid, matches);
                    return Ok(());
                }
                _ => {}
            }

            if C::can_view_board() {
                visitor.view_board(&self.board);
            }

            if C::can_stack_trace() {
                visitor.stack_trace(
                    &self.execution_stack,
                    self.transform_stack.as_slice(),
                    &self.passed_frames,
                );
            }
        }
        Ok(())
    }

    /// Runs an entire scoph program without stopping. Will run the visitor methods on the
    /// specified capabilities.
    ///
    /// NOTE: this function will return after presenting player choices.
    pub fn run<V: Visitor, C: Capabilities>(&mut self, visitor: &mut V) -> Result<(), IError> {
        self.startup(visitor)?;
        self.resume::<V, C>(visitor)
    }

    /// Run an interpreter the whole way through with all capabilities
    pub fn run_full<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), IError> {
        self.run::<V, Full>(visitor)
    }

    /// Check if the game is over and has an outcome
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    pub fn board_size(&self) -> (u32, u32) {
        let (w, h) = self.board.pattern_size();
        (w as u32, h as u32)
    }
}
