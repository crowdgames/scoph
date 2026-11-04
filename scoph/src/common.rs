mod pattern;
mod tree;
mod choice;

use std::{ffi::CString, str::FromStr};

use regex::Replacer;
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};

pub use pattern::{TRRBTPattern, toodee_pattern};
pub use tree::BehaviorTree;
pub use choice::{Rewrite, MatchCoords, PlayerChoices};
#[cfg(test)]
mod test;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub CString);

impl PlayerId {
    /// Replaces all occurrences of [what] w/ [with] and vice-versa.
    pub fn swap_chars(&self, what: &str, with: &str) -> Self {
        let r = swap_chars(self.0.to_str().unwrap(), what, with);
        Self(CString::from_str(r.as_str()).expect("Should replace into a valid string"))
    }
}

impl From<&str> for PlayerId {
    fn from(value: &str) -> Self {
        Self(CString::from_str(value).unwrap())
    }
}

/// Takes a source string and swaps all occurrences of [what] with [with] and vice-versa.
pub fn swap_chars(source: &str, what: &str, with: &str) -> String {
    // NOTE: lot of unwraps.
    let r = regex::Regex::new(&format!("({what}|{with})")).unwrap();

    struct Swapper<'a>(&'a str, &'a str);

    impl<'a> Replacer for Swapper<'a> {
        fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
            if &caps[0] == self.0 {
                dst.push_str(self.1);
            } else {
                dst.push_str(self.0);
            }
        }
    }
    let r = r.replace_all(source, Swapper(what, with));
    String::from_str(&r).unwrap()
}

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
