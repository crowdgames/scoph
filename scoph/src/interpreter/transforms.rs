use crate::common::{NodeAction, TRRBTPattern};

#[derive(Debug, Clone)]
pub enum Transform {
    Ident,
    Rotate,
    Skew,
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
        // TODO: Other nodes with patterns
        _ => Err(TError::Patternless),
    }
}

pub enum TError {
    Patternless,
}

impl Transform {
    pub fn apply(&self, orig: &NodeAction) -> Result<NodeAction, TError> {
        match self {
            Self::Ident => Ok(orig.clone()),
            Self::Rotate => pattern_transform(orig, TRRBTPattern::rotated_90),
            Self::Skew => pattern_transform(orig, TRRBTPattern::skewed),
        }
    }
}

/// Responsible for taking popped nodes and creating transforms
#[derive(Debug, Default)]
pub struct TransformStack(Vec<Transform>);

impl TransformStack {
    pub fn push(&mut self, t: Transform) {
        self.0.push(t);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn apply_transforms(&self, na: NodeAction) -> NodeAction {
        self.0
            .iter()
            .rev()
            .fold(na, |acc, p| p.apply(&acc).unwrap_or(acc))
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
    fn test_pattern_transform() {
        let m = match3x();
        let result = TransformStack(vec![T::Ident, T::Rotate, T::Skew]);
        assert_eq!(
            result.apply_transforms(m),
            NodeAction::Match {
                pattern: three_x().skewed().rotated_90()
            }
        );
    }
}
