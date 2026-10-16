use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};

use crate::common::{Info, NodeAction};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct TRRBTDocument {
    pub name: String,
    pub desc: String,
    pub tree: DocNode,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocInfo {
    #[serde_as(as = NoneAsEmptyString)]
    #[serde(default)]
    nid: Option<String>,

    #[serde_as(as = NoneAsEmptyString)]
    #[serde(default)]
    comment: Option<String>,
}

impl DocInfo {
    pub fn id(&self) -> Option<&str> {
        self.nid.as_ref().map(|s| s.as_str())
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|s| s.as_str())
    }

    fn set_id(&mut self, id: &str) {
        self.nid = Some(id.to_string());
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocNode {
    #[serde(flatten)]
    pub info: DocInfo,

    #[serde(default)]
    pub children: Vec<Self>,

    #[serde(flatten)]
    pub action: NodeAction,
}

impl DocNode {
    pub fn new(action: NodeAction) -> Self {
        Self {
            info: Default::default(),
            children: vec![],
            action,
        }
    }

    pub fn set_id(&mut self, id: &str) {
        self.info.set_id(id);
    }

    pub fn add_child(&mut self, node: DocNode) {
        self.children.push(node);
    }

    pub fn get_child(&self, i: usize) -> Option<&Self> {
        self.children.get(i)
    }
}
