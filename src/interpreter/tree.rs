use petgraph::graph::{DiGraph, NodeIndex};

use crate::{common::{Info, NodeAction}, parse::{Node, TRRBTDocument}};

pub struct BehaviorNode {
    info: Info,
    action: NodeAction,
}

#[derive(Default)]
pub struct BehaviorEdge {
    breakpoint: bool,
}

type TRRBTTree = DiGraph<BehaviorNode, BehaviorEdge>;

// TODO: implement a custom serializer
pub struct BehaviorTree {
    name: String,
    desc: String,
    tree: TRRBTTree,
    root_index: NodeIndex,
}

impl BehaviorTree {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn description(&self) -> &str {
        self.desc.as_str()
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    pub fn get_node_action(&self, node_index: NodeIndex) -> &NodeAction {
        &self.tree[node_index].action
    }

    pub fn get_node_info(&self, node_index: NodeIndex) -> &Info {
        &self.tree[node_index].info
    }

    pub fn get_node_neighbors(&self, node_index: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.tree.neighbors(node_index)
    }
}

fn node_to_behavior_tree_graph(
    n: Node,
    root: &mut Option<NodeIndex>,
    ug: &mut TRRBTTree,
    parent: Option<NodeIndex>,
) {
    let new = BehaviorNode {
        info: n.info,
        action: n.action,
    };

    let new_index = ug.add_node(new);
    if let Some(parent) = parent {
        ug.add_edge(parent, new_index, Default::default());
    } else {
        *root = Some(new_index);
    }

    for child in n.children {
        node_to_behavior_tree_graph(child, root, ug, Some(new_index));
    }
}

impl From<TRRBTDocument> for BehaviorTree {
    fn from(value: TRRBTDocument) -> Self {
        let mut tree = TRRBTTree::new();
        let mut root = None;
        node_to_behavior_tree_graph(value.tree, &mut root, &mut tree, None);

        Self {
            name: value.name,
            desc: value.desc,
            root_index: root.unwrap(),
            tree,
        }
    }
}

