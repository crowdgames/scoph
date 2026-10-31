use derive_more::Debug;
use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use petgraph::graph::{NodeIndex, node_index};

use crate::{
    common::{NodeAction, PlayerId, TRRBTPattern},
    interpreter::{
        BehaviorTree, IError,
        transforms::{Transform, TransformStack},
        visitor::StackFrameInfo,
    },
};

#[derive(Debug)]
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

#[derive(Debug)]
enum Source {
    // node action from a real node in the tree
    Tree {
        #[debug(skip)]
        tree: Rc<BehaviorTree>,
        node_index: NodeIndex,
    },
    // node action from an applied transform to something actually in the tree
    Transform {
        #[debug(skip)]
        tree: Rc<BehaviorTree>,
        transformed: NodeAction,
        original_index: NodeIndex,
    },
}

impl Source {
    fn action(&self) -> &NodeAction {
        match self {
            Self::Tree { tree, node_index } => tree.get_node_action(*node_index),
            Self::Transform { transformed, .. } => transformed,
        }
    }

    fn children(&self) -> impl Iterator<Item = NodeIndex> {
        match self {
            Self::Tree { tree, node_index } => tree.get_node_neighbors(*node_index),
            Self::Transform {
                tree,
                original_index,
                ..
            } => tree.get_node_neighbors(*original_index),
        }
    }

    fn index(&self) -> NodeIndex {
        match self {
            Source::Tree { node_index, .. } => *node_index,
            Source::Transform { original_index, .. } => *original_index,
        }
    }

    fn tree(&self) -> Rc<BehaviorTree> {
        match self {
            Source::Tree { tree, .. } => tree,
            Source::Transform { tree, .. } => tree,
        }
        .clone()
    }
}

/// A frame that lives on the execution stack
#[derive(Debug)]
pub struct ExecutionFrame {
    source: Source,
    child: usize,
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
        Self::from_source(Source::Tree { tree, node_index })
    }

    fn from_source(source: Source) -> Self {
        use NodeAction as N;
        let na = source.action();
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
            source,
            child: 0,
            state,
        }
    }

    pub fn evaluate(
        &mut self,
        board: &mut TRRBTPattern,
        transform_stack: &mut TransformStack,
    ) -> Result<Eval, IError> {
        use crate::common::NodeAction as N;
        match self.source.action() {
            N::Order => {
                let neighbors: Vec<_> = self.source.children().collect();
                if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
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
                let neighbors: Vec<_> = self.source.children().collect();
                if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
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
                let neighbors: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::Finished(false))
                } else if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
                    Ok(Eval::Finished(true))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
            }

            N::Win { pid } => {
                let neighbors: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::End(Outcome::Win(pid.clone())))
                } else if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
                    Ok(Eval::Finished(false))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
            }

            N::All => {
                let neighbors: Vec<_> = self.source.children().collect();
                let StackState::All { all_succeeded } = self.state else {
                    unreachable!()
                };

                Ok(if !all_succeeded {
                    Eval::Finished(false)
                } else if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
                    Eval::Finished(true)
                } else {
                    Eval::Push(neighbors[self.child as usize])
                })
            }

            N::Lose { pid } => {
                let neighbors: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                if !all_failed {
                    Ok(Eval::End(Outcome::Lose(pid.clone())))
                } else if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
                    Ok(Eval::Finished(false))
                } else {
                    Ok(Eval::Push(neighbors[self.child as usize]))
                }
            }

            N::Draw => {
                let neighbors: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };

                Ok(if !all_failed {
                    Eval::End(Outcome::Tie)
                } else if self.child as usize >= neighbors.len() && transform_stack.is_empty() {
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
                let children: Vec<_> = self.source.children().collect();

                let StackState::Transform {
                    ref transforms_list,
                    ref mut transform,
                    ref mut seen_transforms,
                } = self.state
                else {
                    unreachable!();
                };
                loop {
                    if self.child >= children.len() {
                        return Ok(Eval::TransformFinished);
                    }

                    if *transform < transforms_list.len() {
                        transform_stack.push(transforms_list[*transform].clone());
                        *transform += 1;
                        let child_index = children[self.child];
                        let tree = self.source.tree();
                        let child_node = tree.get_node_action(child_index);
                        if !child_node.transformable() {
                            return Ok(Eval::Push(child_index));
                        }
                        let transformed = transform_stack.apply_transforms(child_node.clone());
                        transform_stack.pop();
                        if !seen_transforms.contains(&transformed) {
                            seen_transforms.insert(transformed.clone());
                            return Ok(Eval::TransformPush(Self::from_source(Source::Transform {
                                tree,
                                transformed,
                                original_index: child_index,
                            })));
                        }
                    } else {
                        *transform = 0;
                        self.child += 1;
                    }
                }
            }

            _ => todo!(),
        }
    }

    pub fn child_finished(&mut self, succeeded: bool, untransformed: bool) -> Option<bool> {
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
                if untransformed {
                    self.child += 1;
                }
            }
            S::Order {
                ref mut any_success,
            } => {
                if succeeded {
                    *any_success = true;
                }
                if untransformed {
                    self.child += 1;
                }
            }
            S::None { ref mut all_failed } => {
                if succeeded {
                    *all_failed = false;
                }
                if untransformed {
                    self.child += 1;
                }
            }
            S::All {
                ref mut all_succeeded,
            } => {
                if !succeeded {
                    *all_succeeded = false;
                }
                if untransformed {
                    self.child += 1;
                }
            }
            S::Transform { .. } => {
                return Some(succeeded);
            }
            S::SingleAction => {}
        }

        None
    }

    pub fn source_index(&self) -> NodeIndex {
        self.source.index()
    }

    pub fn info(&self) -> StackFrameInfo {
        todo!("I changed how calls are source so forgive me my poor tests")
    }
}
