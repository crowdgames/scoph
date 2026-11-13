use crate::prelude::*;
use alloc::{collections::BTreeSet, ffi::CString, rc::Rc, str::FromStr};
use derive_more::Debug;
use rand::{Rng, seq::IteratorRandom};

use crate::{
    common::{NodeAction, NodeIndex, PlayerId, TRRBTPattern},
    interpreter::{
        BehaviorTree, IError,
        transforms::{Transform, TransformStack},
        visitor::StackFrameInfo,
    },
};

pub use crate::common::PlayerChoices;

#[derive(Debug, Clone)]
pub enum StackState {
    LoopUntilAll {
        any_success: bool,
        all_failed: bool,
    },
    LoopTimes {
        any_success: bool,
        remaining: u32,
    },
    RandomTry {
        played: BTreeSet<usize>,
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
        random_attempts: Option<BTreeSet<usize>>,
    },
    TransformLink {
        copied: bool,
    },
    SingleAction,
}

impl StackState {
    fn make_transform<const N: usize>(remorig: bool, main: [Transform; N], random: bool) -> Self {
        let mut transforms_list = vec![];
        if !remorig {
            transforms_list.push(Transform::Ident);
        }
        for t in main {
            transforms_list.push(t);
        }
        Self::Transform {
            transforms_list,
            transform: Default::default(),
            random_attempts: random.then(BTreeSet::new),
        }
    }
}

impl<'a> From<&'a Source> for StackState {
    fn from(value: &'a Source) -> Self {
        use NodeAction as N;
        match value.action() {
            N::LoopUntilAll => Self::LoopUntilAll {
                any_success: false,
                all_failed: true,
            },
            N::LoopTimes { times } => Self::LoopTimes {
                any_success: false,
                remaining: *times,
            },
            N::RandomTry => Self::RandomTry {
                played: Default::default(),
                all_failed: true,
            },
            N::Order => Self::Order { any_success: false },
            N::None | N::Win { .. } | N::Lose { .. } | N::Draw => Self::None { all_failed: true },
            N::All => Self::All {
                all_succeeded: true,
            },
            N::SetBoard { .. }
            | N::Rewrite { .. }
            | N::RewriteAll { .. }
            | N::LayerTemplate { .. }
            | N::Match { .. }
            | N::MatchTimes { .. }
            | N::Player { .. }
            | N::AppendRows { .. }
            | N::AppendColumns { .. } => Self::SingleAction,
            &N::XSkew { remorig } => Self::make_transform(remorig, [Transform::Skew], false),
            &N::XRotate { remorig } => Self::make_transform(remorig, [Transform::Rotate90], false),
            &N::XSpin { remorig } => Self::make_transform(
                remorig,
                [
                    Transform::Rotate90,
                    Transform::Rotate180,
                    Transform::Rotate270,
                ],
                false,
            ),
            N::XIdent => Self::make_transform(true, [Transform::Ident], false),
            N::XLink { .. } => Self::TransformLink { copied: false },
            N::XSwap { what, with } => Self::make_transform(
                true,
                [Transform::SwapOnly(what.clone(), with.clone())],
                false,
            ),
            a => {
                todo!("No StackState for {a:?} yet");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Source {
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
#[derive(Debug, Clone)]
pub struct Frame {
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

#[derive(Debug)]
pub enum Eval {
    // behavior tree transforms
    /// indicates a node has finished and provided a result
    Finished(bool),
    /// pushing a node onto the ES from the untransformed tree
    Push(NodeIndex),

    PlayerChoices {
        pid: PlayerId,
        matches: PlayerChoices,
    },
    TransformFinished,
    End(Outcome),
}

impl Frame {
    pub fn from_source(source: Source) -> Self {
        let state = From::from(&source);
        Self {
            source,
            child: 0,
            state,
        }
    }

    fn at_end_of_children(&self, children: &[NodeIndex]) -> bool {
        self.child >= children.len()
    }

    fn short_circuit_eval(
        &self,
        children: &[NodeIndex],
        short_circuit_condition: bool,
        short_circuit: Eval,
        reached_end: Eval,
    ) -> Eval {
        if short_circuit_condition {
            short_circuit
        } else if self.at_end_of_children(children) {
            reached_end
        } else {
            Eval::Push(children[self.child])
        }
    }

    pub(crate) fn evaluate<R>(
        &mut self,
        board: &mut TRRBTPattern,
        transform_stack: &mut TransformStack,
        rng: &mut R,
    ) -> Result<Eval, IError>
    where
        R: Rng,
    {
        use crate::common::NodeAction as N;
        match self.source.action() {
            N::Order => {
                let neighbors: Vec<_> = self.source.children().collect();
                if self.at_end_of_children(&neighbors) {
                    let StackState::Order { any_success } = self.state else {
                        unreachable!();
                    };

                    Ok(Eval::Finished(any_success))
                } else {
                    let child_index = self.child;
                    Ok(Eval::Push(neighbors[child_index]))
                }
            }

            N::LoopUntilAll => {
                let neighbors: Vec<_> = self.source.children().collect();
                if self.at_end_of_children(&neighbors) {
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
                let child_index = self.child;
                Ok(Eval::Push(neighbors[child_index]))
            }

            N::LoopTimes { .. } => {
                let children: Vec<_> = self.source.children().collect();
                if self.at_end_of_children(&children) {
                    let StackState::LoopTimes {
                        any_success,
                        ref mut remaining,
                    } = self.state
                    else {
                        unreachable!();
                    };

                    *remaining -= 1;
                    if *remaining == 0 {
                        return Ok(Eval::Finished(any_success));
                    } else {
                        self.child = 0;
                    }
                }
                Ok(Eval::Push(children[self.child]))
            }

            N::RandomTry => {
                let StackState::RandomTry {
                    ref mut played,
                    all_failed,
                } = self.state
                else {
                    unreachable!()
                };

                Ok(if !all_failed {
                    Eval::Finished(true)
                } else if let Some((new_played, child_index)) = self
                    .source
                    .children()
                    .enumerate()
                    .filter(|(child, _)| !played.contains(child))
                    .choose(rng)
                {
                    played.insert(new_played);
                    Eval::Push(child_index)
                } else {
                    Eval::Finished(false)
                })
            }

            N::None => {
                let children: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                Ok(self.short_circuit_eval(
                    &children,
                    !all_failed,
                    Eval::Finished(false),
                    Eval::Finished(true),
                ))
            }

            N::Win { pid } => {
                let children: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                Ok(self.short_circuit_eval(
                    &children,
                    !all_failed,
                    Eval::End(Outcome::Win(pid.clone())),
                    Eval::Finished(false),
                ))
            }

            N::All => {
                let children: Vec<_> = self.source.children().collect();
                let StackState::All { all_succeeded } = self.state else {
                    unreachable!()
                };

                Ok(self.short_circuit_eval(
                    &children,
                    !all_succeeded,
                    Eval::Finished(false),
                    Eval::Finished(true),
                ))
            }

            N::Lose { pid } => {
                let children: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };
                Ok(self.short_circuit_eval(
                    &children,
                    !all_failed,
                    Eval::End(Outcome::Lose(pid.clone())),
                    Eval::Finished(false),
                ))
            }

            N::Draw => {
                let children: Vec<_> = self.source.children().collect();
                let StackState::None { all_failed } = self.state else {
                    unreachable!()
                };

                Ok(self.short_circuit_eval(
                    &children,
                    !all_failed,
                    Eval::End(Outcome::Tie),
                    Eval::Finished(false),
                ))
            }

            N::SetBoard { pattern } => {
                *board = (*pattern).clone();
                Ok(Eval::Finished(true))
            }

            N::Rewrite { lhs, rhs } => Ok(if let Some((x, y)) = board.matches(lhs).choose(rng) {
                board.rewrite_at(x, y, rhs)?;
                Eval::Finished(true)
            } else {
                Eval::Finished(false)
            }),

            N::RewriteAll { lhs, rhs } => {
                let mut did_rw = false;

                let coords: Vec<_> = board.matches(lhs).collect();
                for (x, y) in coords {
                    board.rewrite_at(x, y, rhs)?;
                    did_rw = true;
                }

                Ok(Eval::Finished(did_rw))
            }

            N::AppendRows { pattern } => {
                board.append_rows(pattern)?;
                Ok(Eval::Finished(true))
            }

            N::AppendColumns { pattern } => {
                board.append_columns(pattern)?;
                Ok(Eval::Finished(true))
            }

            N::LayerTemplate { layer, with } => {
                let (width, height) = board.pattern_size();
                board.set_layer(
                    layer,
                    width,
                    height,
                    vec![CString::from_str(with).unwrap(); width * height],
                );
                Ok(Eval::Finished(true))
            }

            N::Match { pattern } => Ok(Eval::Finished(board.matches(pattern).count() > 0)),

            N::MatchTimes { times, pattern } => Ok(Eval::Finished(
                board.matches(pattern).count() >= *times as usize,
            )),

            N::Player { pid } => {
                let plwr = self.collect_player_rewrites(board, transform_stack, rng)?;
                Ok(if plwr.is_empty() {
                    Eval::Finished(false)
                } else {
                    Eval::PlayerChoices {
                        pid: pid.clone(),
                        matches: plwr,
                    }
                })
            }

            N::XLink { target } => {
                let StackState::TransformLink { ref mut copied } = self.state else {
                    unreachable!()
                };

                Ok(if !*copied {
                    let target_index =
                        self.source
                            .tree()
                            .get_node_by_id(target)
                            .ok_or(IError::BadLink {
                                target: target.clone(),
                            })?;
                    *copied = true;
                    Eval::Push(target_index)
                } else {
                    Eval::TransformFinished
                })
            }

            action if action.is_transform() => {
                let children: Vec<_> = self.source.children().collect();

                let StackState::Transform {
                    ref transforms_list,
                    ref mut transform,
                    .. // TODO: randomness
                } = self.state
                else {
                    unreachable!();
                };

                loop {
                    if self.child >= children.len() {
                        transform_stack.pop();
                        return Ok(Eval::TransformFinished);
                    }

                    let child_index = children[self.child];
                    if *transform < transforms_list.len() {
                        transform_stack.push(transforms_list[*transform].clone());
                        *transform += 1;

                        return Ok(Eval::Push(child_index));
                    } else {
                        // get rid of our old transform and advance to the new one
                        *transform = 0;
                        transform_stack.pop();
                        self.child += 1;
                    }
                }
            }

            _ => todo!(),
        }
    }

    fn collect_player_rewrites<R: Rng>(
        &self,
        board: &mut TRRBTPattern,
        transform_stack: &mut TransformStack,
        rng: &mut R,
    ) -> Result<PlayerChoices, IError> {
        use NodeAction as N;
        let mut matches = PlayerChoices::default();
        let tree = self.source.tree();
        let mut virtual_stack = vec![];
        for child in self.source.children() {
            let checked_action = tree.get_node_action(child);

            loop {
                match checked_action {
                    action if matches!(action, N::Rewrite { .. }) => {
                        let Some(NodeAction::Rewrite { lhs, rhs }) =
                            transform_stack.apply_transforms(action.clone())
                        else {
                            break;
                        };
                        matches.insert((lhs.clone(), rhs.clone()), board.matches(&lhs).collect());
                        virtual_stack.clear();
                        break;
                    }
                    action if action.is_transform() => {
                        if virtual_stack.is_empty() {
                            let transformed = transform_stack
                                .apply_transforms(tree.get_node_action(child).clone())
                                .unwrap();
                            virtual_stack.push(Self::from_source(Source::Transform {
                                tree: tree.clone(),
                                transformed,
                                original_index: child,
                            }));
                        }

                        if let Some(top) = virtual_stack.last_mut() {
                            match top.evaluate(board, transform_stack, rng)? {
                                Eval::Push(node_index) => {
                                    let Some(transformed) = transform_stack
                                        .apply_transforms(tree.get_node_action(node_index).clone())
                                    else {
                                        break;
                                    };

                                    if let NodeAction::Rewrite { lhs, rhs } = &transformed {
                                        matches.insert(
                                            (lhs.clone(), rhs.clone()),
                                            board.matches(lhs).collect(),
                                        );
                                        break;
                                    } else if !transformed.is_transform() {
                                        return Err(IError::BadPlayerChild(
                                            node_index,
                                            transformed,
                                        ));
                                    }

                                    let frame = Self::from_source(Source::Transform {
                                        tree: tree.clone(),
                                        transformed,
                                        original_index: node_index,
                                    });

                                    virtual_stack.push(frame);
                                }
                                Eval::TransformFinished => {
                                    transform_stack.reset_seen();
                                    virtual_stack.pop();
                                }
                                e => unreachable!(
                                    "Non-transform eval result: {e:?}, stack top was {:?}",
                                    top
                                ),
                            }
                        }
                    }
                    a => return Err(IError::BadPlayerChild(child, a.clone())),
                }
            }
        }

        Ok(matches)
    }

    pub fn child_finished(&mut self, succeeded: bool, untransformed: bool) -> Option<bool> {
        use StackState as S;
        match self.state {
            S::SingleAction => return None,
            S::LoopUntilAll {
                ref mut any_success,
                ref mut all_failed,
            } => {
                if succeeded {
                    *any_success = true;
                    *all_failed = false;
                }
            }
            S::LoopTimes {
                ref mut any_success,
                ..
            } => {
                if succeeded {
                    *any_success = true;
                }
            }
            S::RandomTry {
                ref mut all_failed, ..
            } => {
                if succeeded {
                    *all_failed = false;
                }
            }
            S::Order {
                ref mut any_success,
            } => {
                if succeeded {
                    *any_success = true;
                }
            }
            S::None { ref mut all_failed } => {
                if succeeded {
                    *all_failed = false;
                }
            }
            S::All {
                ref mut all_succeeded,
            } => {
                if !succeeded {
                    *all_succeeded = false;
                }
            }
            S::Transform { .. } | S::TransformLink { .. } => {
                return Some(succeeded);
            }
        }

        if untransformed {
            self.child += 1;
        }

        None
    }

    pub fn next_branch(&mut self) {
        self.child += 1;
    }

    pub fn source_index(&self) -> NodeIndex {
        self.source.index()
    }

    pub fn action(&self) -> &NodeAction {
        self.source.action()
    }

    pub fn info(&self) -> StackFrameInfo {
        todo!("I changed how calls are source so forgive me my poor tests")
    }

    pub fn child(&self) -> usize {
        self.child
    }

    pub fn state(&self) -> &StackState {
        &self.state
    }
}
