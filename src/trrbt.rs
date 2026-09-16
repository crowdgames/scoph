use petgraph::Graph;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{NoneAsEmptyString, serde_as};

#[derive(Debug, Serialize, Deserialize)]
pub struct TRRBTDocument {
    pub name: String,
    pub desc: String,
    pub tree: Node,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub nid: Option<String>,

    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub comment: Option<String>,

    #[serde(default)]
    children: Vec<Self>,

    #[serde(flatten)]
    pub action: NodeAction,
}

impl Node {
    pub fn new(action: NodeAction) -> Self {
        Self {
            nid: None,
            comment: None,
            children: vec![],
            action,
        }
    }

    pub fn set_id(&mut self, id: &str) {
        self.nid = Some(id.to_string());
    }

    pub fn add_child(&mut self, node: Node) {
        self.children.push(node);
    }

    pub fn get_child(&self, i: usize) -> Option<&Self> {
        self.children.get(i)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub String);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pattern {
    main: Vec<Vec<String>>,
}

impl Pattern {
    pub fn new(main: Vec<Vec<String>>) -> Self {
        Self { main }
    }

    pub fn filled(cell_pattern: &str, w: usize, h: usize) -> Self {
        let mut main = vec![vec![]; h];
        main.fill_with(|| vec![cell_pattern.to_string(); w]);
        Self { main }
    }
}

fn one_times() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
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
        pattern: Pattern,
    },
    AppendRows {
        pattern: Pattern,
    },
    AppendColumns {
        pattern: Pattern,
    },
    Rewrite {
        lhs: Pattern,
        rhs: Pattern,
    },
    RewriteAll {
        lhs: Pattern,
        rhs: Pattern,
    },
    LayerTemplate {
        layer: String,
        with: String,
    },

    // Condition Nodes,
    Match {
        pattern: Pattern,
    },
    MatchTimes {
        #[serde(default = "one_times")]
        times: u32,
        pattern: Pattern,
    },
}

impl NodeAction {
    pub fn default_x_rotate() -> Self {
        Self::XRotate { remorig: false }
    }

    pub fn default_x_skew() -> Self {
        Self::XSkew { remorig: false }
    }
}
