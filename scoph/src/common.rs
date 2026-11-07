mod choice;
mod pattern;
mod tree;

use std::{ffi::CString, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};

pub use choice::{MatchCoords, PlayerChoices, Rewrite};
pub use pattern::{TRRBTPattern, toodee_pattern};
pub use tree::BehaviorTree;
#[cfg(test)]
mod test;

/// An ID of a player from the TRRBT program.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub CString);

impl PlayerId {
    /// Replaces all occurrences of [what] w/ [with] and vice-versa.
    pub fn swap_chars(&self, what: &str, with: &str) -> Self {
        let r = swap_chars(self.0.to_str().unwrap(), what, with);
        Self(CString::from_str(r.as_str()).expect("Should replace into a valid string"))
    }
}

#[cfg(test)]
mod pid_test {
    use crate::common::PlayerId;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_swap_chars() {
        let x: PlayerId = "X".into();
        assert_eq!(x.swap_chars("X", "Y"), PlayerId::from("Y"));
    }
}

impl From<&str> for PlayerId {
    fn from(value: &str) -> Self {
        Self(CString::from_str(value).unwrap())
    }
}

/// Takes a source string and swaps all occurrences of [what] with [with] and vice-versa.
pub fn swap_chars(source: &str, what: &str, with: &str) -> String {
    let mut out = String::new();

    let mut i = 0;
    while i < source.len() {
        if source.len().checked_sub(what.len()).is_some_and(|l| i <= l)
            && &source[i..(i + what.len())] == what
        {
            out.push_str(with);
            i += what.len();
        } else if source.len().checked_sub(with.len()).is_some_and(|l| i <= l)
            && &source[i..(i + with.len())] == with
        {
            out.push_str(what);
            i += with.len();
        } else {
            out.push_str(&source[i..(i + 1)]);
            i += 1;
        }
    }

    out
}

/// The comment and id of a node from a TRRBT program.
#[serde_as]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Info {
    nid: String,
    #[serde_as(as = NoneAsEmptyString)]
    #[serde(default)]
    comment: Option<String>,
}

impl Info {
    pub fn new(nid: &str) -> Self {
        Self {
            nid: nid.to_string(),
            comment: None,
        }
    }

    pub fn set_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    pub fn set_id(&mut self, id: &str) {
        self.nid = id.to_string();
    }

    pub fn id(&self) -> &str {
        &self.nid
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(String::as_str)
    }
}

fn one_times() -> u32 {
    1
}

#[derive(Debug, strum::Display, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NodeAction {
    // Tree Macros
    XIdent,
    XPrune,
    XMirror {
        #[serde(default)]
        remorig: bool,
    },
    XSkew {
        #[serde(default)]
        remorig: bool,
    },
    XRotate {
        #[serde(default)]
        remorig: bool,
    },
    XSpin {
        #[serde(default)]
        remorig: bool,
    },
    XFlip {
        #[serde(default)]
        remorig: bool,
    },
    XSwap {
        what: String,
        with: String,
    },
    XReplace {
        what: String,
        withs: Vec<String>,
    },
    XUnrollReplace {
        what: String,
    },
    XLink {
        target: String,
    },

    // Player & Win Conditions
    Player {
        pid: PlayerId,
    },
    Win {
        pid: PlayerId,
    },
    Lose {
        pid: PlayerId,
    },
    Draw,

    // control flow nodes
    Order,
    All,
    None,
    RandomTry,
    LoopUntilAll,
    LoopTimes {
        #[serde(default = "one_times")]
        times: u32,
    },

    // Rewrite nodes,
    SetBoard {
        pattern: TRRBTPattern,
    },
    AppendRows {
        pattern: TRRBTPattern,
    },
    AppendColumns {
        pattern: TRRBTPattern,
    },
    Rewrite {
        lhs: TRRBTPattern,
        rhs: TRRBTPattern,
    },
    RewriteAll {
        lhs: TRRBTPattern,
        rhs: TRRBTPattern,
    },
    LayerTemplate {
        layer: String,
        with: String,
    },

    // Condition Nodes,
    Match {
        pattern: TRRBTPattern,
    },
    MatchTimes {
        #[serde(default = "one_times")]
        times: u32,
        pattern: TRRBTPattern,
    },
}

impl NodeAction {
    pub fn default_x_rotate() -> Self {
        Self::XRotate { remorig: false }
    }

    pub fn default_x_skew() -> Self {
        Self::XSkew { remorig: false }
    }

    /// Check if this node action's contents can be changed by a transform node like x-skew,
    /// x-rotate, etc.
    pub fn transformable(&self) -> bool {
        matches!(
            self,
            Self::Rewrite { .. }
                | Self::RewriteAll { .. }
                | Self::Match { .. }
                | Self::MatchTimes { .. }
                | Self::SetBoard { .. }
                | Self::LayerTemplate { .. }
                | Self::AppendRows { .. }
                | Self::AppendColumns { .. }
                | Self::Player { .. }
                | Self::Lose { .. }
                | Self::Win { .. }
        )
    }

    /// Check if this action is a transform.
    pub fn is_transform(&self) -> bool {
        matches!(
            self,
            Self::XSkew { .. }
                | Self::XRotate { .. }
                | Self::XSpin { .. }
                | Self::XIdent
                | Self::XLink { .. }
                | Self::XSwap { .. },
            // TODO: finish others
        )
    }
}
