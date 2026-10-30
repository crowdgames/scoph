use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use petgraph::graph::NodeIndex;

use crate::{
    common::{NodeAction, PlayerId, TRRBTPattern},
    interpreter::{
        BehaviorTree, IError,
        transforms::{Transform, TransformStack},
        visitor::StackFrameInfo,
    },
};

enum StackState {
    LoopUntilAll {
        any_success: bool,
        all_failed: bool,
    },
    Order {
        any_success: bool,
    },
    None {
        all_failed: bool,
    },
    All {
        all_succeeded: bool,
    },
    Transform {
        // list of transforms
        transforms_list: Vec<Transform>,
        // transform index to apply
        transform: usize,
        // help us only keep unique results
        seen_transforms: HashSet<NodeAction>,
    },
    SingleAction,
}

impl StackState {
    fn make_transform(remorig: bool, main: Transform) -> Self {
        let mut transforms_list = vec![];
        if !remorig {
            transforms_list.push(Transform::Ident);
        }
        transforms_list.push(main);
        Self::Transform {
            transforms_list,
            transform: Default::default(),
            seen_transforms: Default::default(),
        }
    }
}

enum CallSource {
    // node action from a real node in the tree
    Tree {
        tree: Rc<BehaviorTree>,
        node_index: NodeIndex,
    },
    // node action from an applied transform to something actually in the tree
    Transform(NodeAction),
}

#[derive(Debug, Default)]
struct ChildlessNode;

impl CallSource {
    fn action(&self) -> &NodeAction {
        match self {
            Self::Tree { tree, node_index } => tree.get_node_action(*node_index),
            Self::Transform(na) => na,
        }
    }

    fn children(&self) -> Result<impl Iterator<Item = NodeIndex>, ChildlessNode> {
        match self {
            Self::Tree { tree, node_index } => Ok(tree.get_node_neighbors(*node_index)),
            _ => Err(ChildlessNode),
        }
    }
}

/// A frame that lives on the execution stack
pub struct ExecutionFrame {
    source: CallSource,
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
    // behavior tree transforms
    /// indicates a node has finished and provided a result
    Finished(bool),
    /// pushing a node onto the ES from the untransformed tree
    Push(NodeIndex),

    // special evals for transforms
    /// Push a
    TransformPush(ExecutionFrame),
    TransformFinished,
    End(Outcome),
}

impl ExecutionFrame {
    pub fn new(tree: Rc<BehaviorTree>, node_index: NodeIndex) -> Self {
        use NodeAction as N;
        let na = tree.get_node_action(node_index);
        let state = match na {
            N::LoopUntilAll => StackState::LoopUntilAll {
                any_success: false,
                all_failed: true,
            },
            N::Order => StackState::Order { any_success: false },
            N::None | N::Win { .. } | N::Lose { .. } | N::Draw => {
                StackState::None { all_failed: true }
            }
            N::All => StackState::All {
                all_succeeded: true,
            },
            N::SetBoard { .. } | N::Rewrite { .. } | N::Match { .. } | N::MatchTimes { .. } => {
                StackState::SingleAction
            }
            &N::XSkew { remorig } => StackState::make_transform(remorig, Transform::Skew),
            &N::XRotate { remorig } => StackState::make_transform(remorig, Transform::Rotate),
            _ => {
                todo!()
            }
        };

        Self {
            source: CallSource::Tree { node_index, tree },
            child: 0,
            state,
        }
    }

    const CHILDLESS_ERROR_MESSAGE: &str =
        "Tried to the get the children of nodes from a source that doesn't have children";

    pub fn evaluate(
        &mut self,
        board: &mut TRRBTPattern,
        transform_stack: &mut TransformStack,
    ) -> Result<Eval, IError> {
        use crate::common::NodeAction as N;
        match self.source.action() {
            N::Order => {
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
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
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
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
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
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
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
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

            N::All => {
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
                let StackState::All { all_succeeded } = self.state else {
                    unreachable!()
                };

                Ok(if !all_succeeded {
                    Eval::Finished(false)
                } else if self.child as usize >= neighbors.len() {
                    Eval::Finished(true)
                } else {
                    Eval::Push(neighbors[self.child as usize])
                })
            }

            N::Lose { pid } => {
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
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

            N::Draw => {
                let neighbors: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };

                Ok(if !all_failed {
                    Eval::End(Outcome::Tie)
                } else if self.child as usize >= neighbors.len() {
                    Eval::Finished(false)
                } else {
                    Eval::Push(neighbors[self.child as usize])
                })
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

            N::Match { pattern } => Ok(Eval::Finished(board.matches(pattern).count() > 0)),

            N::MatchTimes { times, pattern } => {
                let mut matched_all = true;

                for (_, coords) in board.matches(pattern) {
                    if coords.count() != *times as usize {
                        matched_all = false;
                        break;
                    }
                }

                Ok(Eval::Finished(matched_all))
            }

            action if action.is_transform() => {
                let children: Vec<_> = self
                    .source
                    .children()
                    .expect(Self::CHILDLESS_ERROR_MESSAGE)
                    .collect();
                Ok(if self.child as usize >= children.len() {
                    Eval::TransformFinished
                } else if let Some(transformed) = transform_stack.pop_front_source_list() {
                    todo!()
                } else {
                    todo!()
                })
            }

            _ => todo!(),
        }
    }

    pub fn child_finished(&mut self, succeeded: bool) -> Option<bool> {
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
            S::All {
                ref mut all_succeeded,
            } => {
                if !succeeded {
                    *all_succeeded = false;
                }
                self.child += 1;
            }
            S::Transform { .. } => return Some(succeeded),
            S::SingleAction => {}
        }

        None
    }

    pub fn info(&self) -> StackFrameInfo {
        todo!("I changed how calls are source so forgive me my poor tests")
    }
}
