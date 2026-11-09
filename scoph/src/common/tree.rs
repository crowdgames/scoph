use std::{
    ffi::{CStr, CString},
    path::Path,
    str::FromStr,
};

use crate::{
    common::{Info, NodeAction},
    interpreter::IError,
    parse::{DocNode, TRRBTDocument},
};
use ::serde::Deserialize;
use anyhow::anyhow;
use petgraph::graph::{DiGraph, NodeIndex};

#[cfg(test)]
mod test;
// mod serde;

#[derive(Debug, Deserialize)]
pub struct BehaviorNode {
    #[serde(flatten)]
    info: Info,
    #[serde(flatten)]
    action: NodeAction,
}

#[derive(Debug, Default)]
pub struct BehaviorEdge;

pub(crate) type TRRBTTree = DiGraph<BehaviorNode, BehaviorEdge>;

// TODO: implement a custom serializer for BehaviorTree

/// An immutable structure that represents a TRRBT program.
/// They can be deserialized a from json files.
#[derive(Debug)]
pub struct BehaviorTree {
    name: CString,
    desc: CString,
    tree: TRRBTTree,
    root_index: NodeIndex,
}

impl BehaviorTree {
    /// Finds a json file at the provided path and loads it
    pub fn load_from_json_file<T: AsRef<Path>>(path: T) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::load_from_json_text(s.as_str()).map_err(|e| anyhow!("{}", e))
    }

    /// Loads a tree from json text directly
    pub fn load_from_json_text(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text).map(|t: TRRBTDocument| t.into())
    }

    /// Takes all transformations in a tree and applies them into a new [BehaviorTree].
    pub fn expand(self) -> Result<Self, IError> {
        todo!()
    }

    /// Gets the name of the game
    pub fn name(&self) -> &CStr {
        self.name.as_c_str()
    }

    /// Gets the description of the game
    pub fn description(&self) -> &CStr {
        self.desc.as_c_str()
    }

    /// Returns the petgraph index of the root node of the tree
    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    /// Gets a ref to the node action of the provided node action in the tree
    pub fn get_node_action(&self, node_index: NodeIndex) -> &NodeAction {
        &self.tree[node_index].action
    }

    /// Returns the info of the node in the graph at the provided petgraph index
    pub fn get_node_info(&self, node_index: NodeIndex) -> &Info {
        &self.tree[node_index].info
    }

    /// Searches the tree for the petgraph index of the node with the given node ID
    pub fn get_node_by_id(&self, id: &str) -> Option<NodeIndex> {
        self.tree
            .node_indices()
            .find_map(|i| (self.tree[i].info.id() == id).then_some(i))
    }

    /// Gets the outgoing connections of the provided node in order
    pub fn get_node_neighbors(&self, node_index: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.tree
            .neighbors(node_index)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
    }

    /// Gets the child of a node by index
    pub fn get_node_child(&self, node_index: NodeIndex, child: u32) -> Option<NodeIndex> {
        self.get_node_neighbors(node_index).nth(child as usize)
    }
}

fn node_to_behavior_tree_graph(
    n: DocNode,
    root: &mut Option<NodeIndex>,
    index_base: &mut u32,
    ug: &mut TRRBTTree,
    parent: Option<NodeIndex>,
) {
    let if_no_id = format!("{}", *index_base);
    let mut info = Info::new(n.info.id().unwrap_or_else(|| {
        *index_base += 1;
        &if_no_id
    }));

    if let Some(comment) = n.info.comment() {
        info = info.set_comment(comment);
    }

    let new = BehaviorNode {
        info,
        action: n.action,
    };

    let new_index = ug.add_node(new);
    if let Some(parent) = parent {
        ug.add_edge(parent, new_index, Default::default());
    } else {
        *root = Some(new_index);
    }

    for child in n.children {
        node_to_behavior_tree_graph(child, root, index_base, ug, Some(new_index));
    }
}

impl From<TRRBTDocument> for BehaviorTree {
    fn from(value: TRRBTDocument) -> Self {
        let mut tree = TRRBTTree::new();
        let mut root = None;
        let mut index_base = 0;
        node_to_behavior_tree_graph(value.tree, &mut root, &mut index_base, &mut tree, None);

        Self {
            name: CString::from_str(&value.name).expect("Valid name"),
            desc: CString::from_str(&value.desc).expect("Valid description"),
            root_index: root.unwrap(),
            tree,
        }
    }
}

