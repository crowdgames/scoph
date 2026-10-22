use std::ffi::CStr;

use petgraph::{adj::Neighbors, graph::NodeIndex};
use serde_with::rust::unwrap_or_skip;

use crate::{
    common::{NodeAction, PlayerId, TRRBTPattern},
    interpreter::{BehaviorTree, IError, visitor::StackFrameInfo},
};

enum StackState {
    LoopUntilAll { any_success: bool, all_failed: bool },
    Order { any_success: bool },
    None { all_failed: bool },
    SingleAction,
}

/// Signifies that the provided action is not a transform node
#[derive(Debug)]
pub struct NonTransformNode;

pub enum Transform {
    Ident,
    Rotate { remorig: bool },
    Skew { remorig: bool },
    SwapOnly { what: String, with: String },
    Link { target_id: String },
}

impl TryFrom<&NodeAction> for Transform {
    type Error = NonTransformNode;

    fn try_from(value: &NodeAction) -> Result<Self, Self::Error> {
        match value {
            &NodeAction::XSkew { remorig } => Ok(Self::Skew { remorig }),
            &NodeAction::XIdent => Ok(Self::Ident),
            &NodeAction::XRotate { remorig } => Ok(Self::Rotate { remorig }),
            NodeAction::XSwap { what, with } => Ok(Self::SwapOnly {
                what: what.to_owned(),
                with: with.to_owned(),
            }),
            NodeAction::XLink { target } => Ok(Self::Link {
                target_id: target.to_owned(),
            }),
            _ => Err(NonTransformNode),
        }
    }
}

pub struct Call {
    node_index: NodeIndex,
    child: u32,
    state: StackState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Win(PlayerId),
    Lose(PlayerId),
    Tie,
}

pub enum Eval {
    Finished(bool),
    Push(NodeIndex),
    End(Outcome),
}

impl Call {
    pub fn new(tree: &BehaviorTree, node_index: NodeIndex) -> Self {
        use NodeAction as N;
        let state = match tree.get_node_action(node_index) {
            N::LoopUntilAll => StackState::LoopUntilAll {
                any_success: false,
                all_failed: true,
            },
            N::Order => StackState::Order { any_success: false },
            N::None | N::Win { .. } | N::Lose { .. } => StackState::None { all_failed: true },
            N::SetBoard { .. } | N::Rewrite { .. } => StackState::SingleAction,
            _ => todo!(),
        };

        Self {
            node_index,
            child: 0,
            state,
        }
    }

    pub fn evaluate(
        &mut self,
        tree: &BehaviorTree,
        board: &mut TRRBTPattern,
    ) -> Result<Eval, IError> {
        use crate::common::NodeAction as N;
        match tree.get_node_action(self.node_index) {
            N::Order => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                if self.child as usize >= neighbors.len() {
                    let StackState::Order { any_success } = self.state else {
                        unreachable!();
                    };

                    Ok(Eval::Finished(any_success))
                } else {
                    let child_index = self.child as usize;
                    Ok(Eval::Push(neighbors[child_index]))
                }
            }

            N::LoopUntilAll => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                if self.child as usize >= neighbors.len() {
                    let StackState::LoopUntilAll {
                        any_success,
                        ref mut all_failed,
                    } = self.state
                    else {
                        unreachable!();
                    };

                    if *all_failed {
                        return Ok(Eval::Finished(any_success));
                    } else {
                        self.child = 0;
                        *all_failed = true;
                    }
                }
                let child_index = self.child as usize;
                Ok(Eval::Push(neighbors[child_index]))
            }

            N::None => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::Finished(false))
                } else if self.child as usize >= neighbors.len() {
                    Ok(Eval::Finished(true))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
            }

            N::Win { pid } => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::End(Outcome::Win(pid.clone())))
                } else if self.child as usize >= neighbors.len() {
                    Ok(Eval::Finished(false))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
            }

            N::Lose { pid } => {
                let neighbors: Vec<_> = tree.get_node_neighbors(self.node_index).collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::End(Outcome::Lose(pid.clone())))
                } else if self.child as usize >= neighbors.len() {
                    Ok(Eval::Finished(false))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
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

    pub fn child_finished(&mut self, succeeded: bool) {
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
                self.child += 1;
            }
            S::Order {
                ref mut any_success,
            } => {
                if succeeded {
                    *any_success = true;
                }
                self.child += 1;
            }
            S::None { ref mut all_failed } => {
                if succeeded {
                    *all_failed = false;
                }
                self.child += 1;
            }
            S::SingleAction => {}
        }
    }

    pub fn info(&self) -> StackFrameInfo {
        (self.node_index, self.child)
    }
}
