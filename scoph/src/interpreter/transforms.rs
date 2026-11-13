use crate::common::{BehaviorTree, NodeAction, PlayerId, TRRBTPattern};
use crate::prelude::*;
use alloc::rc::Rc;
use ordermap::OrderSet;

type HashSet<T> = OrderSet<T, fnv::FnvBuildHasher>;

#[derive(Debug, Clone)]
pub enum Transform {
    Ident,
    Rotate90,
    Rotate180,
    Rotate270,
    Skew,
    SwapOnly(String, String),
    Link(Rc<BehaviorTree>, String),
}

fn pattern_transform(
    na: &NodeAction,
    xform: impl Fn(&TRRBTPattern) -> TRRBTPattern,
) -> Result<NodeAction, TError> {
    use NodeAction as N;
    match na {
        N::Match { pattern } => Ok(N::Match {
            pattern: xform(pattern),
        }),
        N::Rewrite { lhs, rhs } => Ok(N::Rewrite {
            lhs: xform(lhs),
            rhs: xform(rhs),
        }),
        N::RewriteAll { lhs, rhs } => Ok(N::RewriteAll {
            lhs: xform(lhs),
            rhs: xform(rhs),
        }),
        N::SetBoard { pattern } => Ok(N::SetBoard {
            pattern: xform(pattern),
        }),
        N::AppendRows { pattern } => Ok(N::AppendRows {
            pattern: xform(pattern),
        }),
        N::AppendColumns { pattern } => Ok(N::AppendColumns {
            pattern: xform(pattern),
        }),
        _ => Err(TError::Patternless),
    }
}

fn chars_transform(
    na: &NodeAction,
    pid_xform: impl Fn(&PlayerId) -> PlayerId,
    pattern_xform: impl Fn(&TRRBTPattern) -> TRRBTPattern,
) -> Result<NodeAction, TError> {
    use NodeAction as N;
    match na {
        N::Match { pattern } => Ok(N::Match {
            pattern: pattern_xform(pattern),
        }),
        N::Rewrite { lhs, rhs } => Ok(N::Rewrite {
            lhs: pattern_xform(lhs),
            rhs: pattern_xform(rhs),
        }),
        N::RewriteAll { lhs, rhs } => Ok(N::RewriteAll {
            lhs: pattern_xform(lhs),
            rhs: pattern_xform(rhs),
        }),
        N::SetBoard { pattern } => Ok(N::SetBoard {
            pattern: pattern_xform(pattern),
        }),
        N::AppendRows { pattern } => Ok(N::AppendRows {
            pattern: pattern_xform(pattern),
        }),
        N::AppendColumns { pattern } => Ok(N::AppendColumns {
            pattern: pattern_xform(pattern),
        }),
        N::Win { pid } => Ok(N::Win {
            pid: pid_xform(pid),
        }),
        N::Lose { pid } => Ok(N::Lose {
            pid: pid_xform(pid),
        }),
        N::Player { pid } => Ok(N::Player {
            pid: pid_xform(pid),
        }),
        _ => Err(TError::Charless),
    }
}

pub enum TError {
    Patternless,
    Charless,
    BadLink,
}

impl Transform {
    pub fn apply(&self, orig: &NodeAction) -> Result<NodeAction, TError> {
        match self {
            Self::Ident => Ok(orig.clone()),
            Self::Rotate90 => pattern_transform(orig, TRRBTPattern::rotated_90),
            Self::Rotate180 => pattern_transform(orig, |pattern| pattern.rotated_90().rotated_90()),
            Self::Rotate270 => pattern_transform(orig, |pattern| {
                pattern.rotated_90().rotated_90().rotated_90()
            }),
            Self::Skew => pattern_transform(orig, TRRBTPattern::skewed),
            Self::SwapOnly(what, with) => chars_transform(
                orig,
                |pid| pid.swap_chars(what, with),
                |pattern| pattern.swap_chars(what, with),
            ),
            Self::Link(tree, target_id) => {
                let target_index = tree.get_node_by_id(target_id).ok_or(TError::BadLink)?;
                Ok(tree.get_node_action(target_index).clone())
            }
        }
    }
}

/// Responsible for taking popped nodes and creating transforms
#[derive(Debug, Default)]
pub struct TransformStack {
    // NOTE: could potentially optimize this by using Rc's in a StackState's transform list
    applications: Vec<Transform>,
    seen: HashSet<NodeAction>,
}

impl TransformStack {
    pub fn push(&mut self, t: Transform) {
        self.applications.push(t);
    }

    pub fn reset_seen(&mut self) {
        self.seen.clear();
    }

    pub fn pop(&mut self) {
        self.applications.pop();
    }

    pub fn apply_transforms(&mut self, na: NodeAction) -> Option<NodeAction> {
        let result = self
            .applications
            .iter()
            .rev()
            .fold(na, |acc, p| p.apply(&acc).unwrap_or(acc));

        if !self.seen.contains(&result) {
            self.seen.insert(result.clone());
            Some(result)
        } else {
            None
        }
    }

    pub(crate) fn as_slice(&self) -> &[Transform] {
        self.applications.as_slice()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.applications.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::common::{TRRBTPattern, toodee_pattern};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ttt_o_tree() {
        let mut stack = TransformStack {
            applications: vec![
                Transform::SwapOnly("X".to_string(), "O".to_string()),
                Transform::Ident,
            ],
            seen: Default::default(),
        };

        let orw = NodeAction::Rewrite {
            lhs: TRRBTPattern::from([("main", [["O"]])]),
            rhs: TRRBTPattern::from([("main", [["_"]])]),
        };

        assert_eq!(
            stack.apply_transforms(NodeAction::Rewrite {
                lhs: TRRBTPattern::from([("main", [["X"]])]),
                rhs: TRRBTPattern::from([("main", [["_"]])])
            }),
            Some(orw.clone())
        );

        assert_eq!(&stack.seen, &HashSet::from_iter([orw]));
    }
}
