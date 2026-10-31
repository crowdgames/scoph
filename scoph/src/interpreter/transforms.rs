use crate::common::{NodeAction, TRRBTPattern};

#[derive(Debug, Clone)]
pub enum Transform {
    Ident,
    Rotate90,
    Rotate180,
    Rotate270,
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

pub enum TError {
    Patternless,
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
        let result = TransformStack(vec![T::Ident, T::Rotate90, T::Skew]);
        assert_eq!(
            result.apply_transforms(m),
            NodeAction::Match {
                pattern: three_x().skewed().rotated_90()
            }
        );
    }
}
