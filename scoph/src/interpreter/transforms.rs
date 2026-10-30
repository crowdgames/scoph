use std::collections::VecDeque;

use crate::{common::{NodeAction, TRRBTPattern}, interpreter::stack::ExecutionFrame};

/// Signifies that the provided action is not a transform node
#[derive(Debug)]
pub struct NonTransformNode;

#[derive(Debug)]
pub enum Transform {
    Ident,
    Rotate,
    Skew,
}

fn pattern_transform(na: NodeAction, xform: impl Fn(&TRRBTPattern) -> TRRBTPattern) -> Result<NodeAction, TError> {
    use NodeAction as N;
    match na {
        N::Match { pattern } => Ok(N::Match { pattern: xform( &pattern ) }),
        _ => Err(TError::Patternless),
    }
}

pub enum TError {
    Patternless,
}


impl Transform {
    pub fn apply(&self, orig: NodeAction) -> Result<NodeAction, TError> {
        match self {
            Transform::Ident => Ok(orig),
            Transform::Rotate  => {
                pattern_transform(orig, TRRBTPattern::rotated_90)
            }
            Transform::Skew  => pattern_transform(orig, TRRBTPattern::skewed),
        }
    }
}

/// Responsible for taking popped nodes and creating transforms
#[derive(Debug, Default)]
pub struct TransformStack {
    stack: Vec<Transform>,
}

impl TransformStack {
    pub fn push(&mut self, t: Transform) {
        self.stack.push(t);
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
}
