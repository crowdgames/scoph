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

#[cfg(test)]
mod test;

mod serde;

pub type NodeIndex = usize;
type EdgeIndex = usize;

#[derive(Debug)]
struct Node {
    action: NodeAction,
    info: Info,
    first: Option<EdgeIndex>,
    last: Option<EdgeIndex>,
}

#[derive(Debug)]
struct Edge {
    target: NodeIndex,
    next: Option<EdgeIndex>,
}

// from https://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
#[derive(Default, Debug)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    fn add_node(&mut self, action: NodeAction, info: Info) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node {
            action,
            info,
            first: None,
            last: None,
        });
        index
    }

    fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        let edge_index = self.edges.len();
        self.edges.push(Edge { target, next: None });
        let node_data = &mut self.nodes[source];
        match node_data.last {
            Some(last) => self.edges[last].next = Some(edge_index),
            None => node_data.first = Some(edge_index),
        }
        node_data.last = Some(edge_index)
    }

    pub fn children(&self, source: NodeIndex) -> Successors<'_> {
        let first = self.nodes[source].first;
        Successors {
            graph: self,
            current: first,
        }
    }
}

pub struct Successors<'graph> {
    graph: &'graph Graph,
    current: Option<EdgeIndex>,
}

impl<'graph> Iterator for Successors<'graph> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let edge = &self.graph.edges[self.current?];
        self.current = edge.next;
        Some(edge.target)
    }
}

/// An immutable structure that represents a TRRBT program.
/// They can be deserialized a from json files.
#[derive(Debug)]
pub struct BehaviorTree {
    name: CString,
    desc: CString,
    tree: Graph,
}

impl BehaviorTree {
    /// Finds a json file at the provided path and loads it
    pub fn load_from_json_file<T: AsRef<Path>>(path: T) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::load_from_json_text(s.as_str()).map_err(|e| anyhow!("{}", e))
    }

    /// Loads a tree from json text directly
    pub fn load_from_json_text(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text).map(|td: TRRBTDocument| td.into())
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
        0
    }

    /// Gets a ref to the node action of the provided node action in the tree
    pub fn get_node_action(&self, node_index: NodeIndex) -> &NodeAction {
        &self.tree.nodes[node_index].action
    }

    /// Returns the info of the node in the graph at the provided petgraph index
    pub fn get_node_info(&self, node_index: NodeIndex) -> &Info {
        &self.tree.nodes[node_index].info
    }

    /// Searches the tree for the petgraph index of the node with the given node ID
    pub fn get_node_by_id(&self, id: &str) -> Option<NodeIndex> {
        self.tree
            .nodes
            .iter()
            .enumerate()
            .find_map(|(index, n)| n.info.id().eq(id).then_some(index))
    }

    /// Gets the outgoing connections of the provided node in order
    pub fn get_node_neighbors(&self, node_index: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.tree.children(node_index)
    }

    /// Gets the child of a node by index
    pub fn get_node_child(&self, node_index: NodeIndex, child: u32) -> Option<NodeIndex> {
        self.get_node_neighbors(node_index).nth(child as usize)
    }
}

fn node_to_behavior_tree_graph(
    n: DocNode,
    index_base: &mut u32,
    ug: &mut Graph,
    parent: Option<NodeIndex>,
) {
    let id = n.info.id();
    let if_no_id = format!("{}", index_base);
    let mut info = Info::new(if id.is_empty() {
        *index_base += 1;
        &if_no_id
    } else {
        id
    });

    if let Some(comment) = n.info.comment() {
        info = info.set_comment(comment);
    }

    let new_index = ug.add_node(n.action, info);
    if let Some(parent) = parent {
        ug.add_edge(parent, new_index);
    }

    for child in n.children {
        node_to_behavior_tree_graph(child, index_base, ug, Some(new_index));
    }
}

impl From<TRRBTDocument> for BehaviorTree {
    fn from(value: TRRBTDocument) -> Self {
        let mut tree = Graph::default();
        let mut index_base = 0;
        node_to_behavior_tree_graph(value.tree, &mut index_base, &mut tree, None);

        Self {
            name: value.name,
            desc: value.desc,
            tree,
        }
    }
}
