use std::collections::VecDeque;

use crate::common::{NodeAction, TRRBTPattern};

/// Signifies that the provided action is not a transform node
#[derive(Debug)]
pub struct NonTransformNode;

#[derive(Debug)]
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

/// Transform-related errors
pub enum TError {
    /// Tried applying a pattern transform on something with no pattern
    Patternless,
    /// Tried applying a chars transform on something without any chars
    Charless,
}

/// The result of a [Transform] being applied to a [NodeAction]
pub enum Transformed {
    /// The transformed node will replace the original in the tree
    Remorig(NodeAction),
    /// The transformed node will follow the original in the tree
    Keeporig(NodeAction, NodeAction),
    /// The transform doesn't change any nodes
    None,
}

fn pattern_transform<F>(
    orig: NodeAction,
    remorig: bool,
    transform_fn: F,
) -> Result<Transformed, TError>
where
    F: Fn(&TRRBTPattern) -> TRRBTPattern,
{
    use NodeAction as N;
    use TError as T;
    use Transformed::{Keeporig as K, Remorig as R};
    match &orig {
        N::Match { pattern } => {
            let rotated = N::Match {
                pattern: transform_fn(pattern),
            };
            Ok(if remorig {
                R(rotated)
            } else {
                K(orig, rotated)
            })
        }
        _ => Err(T::Patternless),
    }
}

impl Transform {
    pub fn apply(&self, orig: NodeAction) -> Result<Transformed, TError> {
        match self {
            Transform::Ident => Ok(Transformed::None),
            &Transform::Rotate { remorig } => {
                pattern_transform(orig, remorig, TRRBTPattern::rotated_90)
            }
            &Transform::Skew { remorig } => pattern_transform(orig, remorig, TRRBTPattern::skewed),
            Transform::SwapOnly { what, with } => todo!(),
            Transform::Link { target_id } => todo!(),
        }
    }
}

/// Responsible for taking popped nodes and creating transforms
#[derive(Debug, Default)]
pub struct TransformStack {
    transforms_list: Vec<Transform>,
    source_list: VecDeque<NodeAction>,
}

impl TransformStack {
    pub fn fresh(&self) -> bool {
        !self.transforms_list.is_empty() && self.source_list.is_empty()
    }

    pub fn populate_source_list(&mut self, na: NodeAction) {
        assert!(na.transformable());

        let mut sl = VecDeque::from([na]);
        for t in self.transforms_list.iter().rev() {
            sl = sl
                .into_iter()
                .filter_map(|action| t.apply(action).ok())
                .fold(VecDeque::new(), |mut sl, transformed| {
                    match transformed {
                        Transformed::None => {}
                        Transformed::Remorig(new) => {
                            if !sl.contains(&new) {
                                sl.push_back(new);
                            }
                        }
                        Transformed::Keeporig(orig, new) => {
                            if !sl.contains(&new) {
                                sl.push_back(orig);
                            }
                            if !sl.contains(&new) {
                                sl.push_back(new);
                            }
                        }
                    }
                    sl
                });
        }
        self.source_list = sl;
    }

    pub fn pop_front_source_list(&mut self) -> Option<NodeAction> {
        self.source_list.pop_front()
    }

    pub fn push(&mut self, t: Transform) {
        self.transforms_list.push(t);
    }

    pub fn pop_transform(&mut self) {
        self.transforms_list.pop();
    }
}

#[cfg(test)]
mod test {
    use crate::common::{TRRBTPattern, toodee_pattern};

    use super::*;
    use Transform as T;
    use pretty_assertions::assert_eq;

    fn three_x() -> TRRBTPattern {
        TRRBTPattern::from([("main", toodee_pattern(3, 1, ["X", "X", "X"]))])
    }

    fn match3x() -> NodeAction {
        NodeAction::Match { pattern: three_x() }
    }

    #[test]
    fn test_transform_population() {
        let mut ts = TransformStack {
            transforms_list: vec![T::Rotate { remorig: false }, T::Skew { remorig: false }],
            source_list: Default::default(),
        };

        assert!(ts.fresh());

        ts.populate_source_list(match3x());

        assert_eq!(
            ts.source_list,
            vec![
                match3x(),
                NodeAction::Match {
                    pattern: three_x().rotated_90()
                },
                NodeAction::Match {
                    pattern: three_x().skewed()
                },
                NodeAction::Match {
                    pattern: three_x().skewed().rotated_90()
                },
            ]
        );

        assert!(!ts.fresh());
    }

    #[test]
    fn test_transform_population2() {
        let mut ts = TransformStack {
            transforms_list: vec![T::Skew { remorig: false }, T::Rotate { remorig: false }],
            source_list: Default::default(),
        };

        assert!(ts.fresh());

        ts.populate_source_list(match3x());

        assert_eq!(
            ts.source_list,
            vec![
                match3x(),
                NodeAction::Match {
                    pattern: three_x().skewed()
                },
                NodeAction::Match {
                    pattern: three_x().rotated_90()
                },
            ]
        );

        assert!(!ts.fresh());
    }
}
