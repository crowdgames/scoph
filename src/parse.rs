use serde::{Deserialize, Serialize};
use serde_with::{serde_as};

use crate::{BehaviorTree, common::{Info, NodeAction, Pattern, PlayerId}};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct TRRBTDocument {
    pub name: String,
    pub desc: String,
    pub tree: Node,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    #[serde(flatten)]
    pub info: Info,

    #[serde(default)]
    pub children: Vec<Self>,

    #[serde(flatten)]
    pub action: NodeAction,
}

impl Node {
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

    pub fn add_child(&mut self, node: Node) {
        self.children.push(node);
    }

    pub fn get_child(&self, i: usize) -> Option<&Self> {
        self.children.get(i)
    }
}



