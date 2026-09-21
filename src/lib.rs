pub mod parse;
// common types across TRRBT
pub mod common;

use common::{Info, NodeAction};
use petgraph::graph::{NodeIndex, UnGraph};
use toodee::TooDee;

use crate::parse::{Node, TRRBTDocument};

pub struct BehaviorNode {
    info: Info,
    action: NodeAction,
}

#[derive(Default)]
pub struct BehaviorEdge {
    breakpoint: bool,
}

// TODO: implement a custom serializer
pub struct BehaviorTree {
    name: String,
    desc: String,
    tree: UnGraph<BehaviorNode, BehaviorEdge>,
}

fn node_to_behavior_tree_graph(
    n: Node,
    ug: &mut UnGraph<BehaviorNode, BehaviorEdge>,
    parent: Option<NodeIndex>,
) {
    let new = BehaviorNode {
        info: n.info,
        action: n.action,
    };

    let new_index = ug.add_node(new);
    if let Some(parent) = parent {
        ug.add_edge(parent, new_index, Default::default());
    }

    for child in n.children {
        node_to_behavior_tree_graph(child, ug, Some(new_index));
    }
}

impl From<TRRBTDocument> for BehaviorTree {
    fn from(value: TRRBTDocument) -> Self {
        let mut tree = UnGraph::new_undirected();
        node_to_behavior_tree_graph(value.tree, &mut tree, None);

        Self {
            name: value.name,
            desc: value.desc,
            tree,
        }
    }
}

struct Call {
    ix: NodeIndex,
}

pub struct Interpreter {
    tree: BehaviorTree,
    board: TooDee<String>,
    call_stack: Vec<Call>,
}
