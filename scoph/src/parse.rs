use alloc::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::common::{Info, NodeAction};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct TRRBTDocument {
    pub name: CString,
    pub desc: CString,
    pub tree: DocNode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocNode {
    #[serde(flatten)]
    pub info: Info,

    #[serde(flatten)]
    pub action: NodeAction,

    #[serde(default)]
    pub children: Vec<Self>,
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

use crate::prelude::*;
