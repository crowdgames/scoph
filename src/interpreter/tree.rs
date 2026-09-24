use crate::{
    common::{Info, NodeAction},
    parse::{Node, TRRBTDocument},
};
use anyhow::anyhow;
use petgraph::graph::{DiGraph, NodeIndex};

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
    pub fn load_from_json_file(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::load_from_json_text(s.as_str()).map_err(|e| anyhow!("{}", e))
    }

    pub fn load_from_json_text(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text).map(|t: TRRBTDocument| t.into())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn description(&self) -> &str {
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

#[cfg(test)]
mod test {
    use crate::common::{TRRBTPattern, toodee_pattern};

    use super::*;
    use anyhow::Result;
    use toodee::TooDee;

    #[test]
    fn test_loading_from_text() -> Result<()> {
        const SIMPLE_GAME: &str = r#"{
            "name" : "A simple game",
            "desc" : "From a simpler time",
            "tree": {
                "type" : "loop-until-all",
                "nid" : "",
                "comment" : "A lame one-node loop",
                "children" : [ 
                    {
                        "type" : "set-board",
                        "nid": "",
                        "comment": "",
                        "pattern" : { 
                            "main" : [
                                ["A", "B", "C"],
                                ["D", "E", "F"],
                                ["G", "H", "I"]
                            ] 
                        }
                    }
                ]
            }
        }"#;

        let behavior_tree = BehaviorTree::load_from_json_text(SIMPLE_GAME)?;
        assert_eq!(behavior_tree.name(), "A simple game");
        assert_eq!(behavior_tree.description(), "From a simpler time");

        let map_to_node_action = |n: NodeIndex| behavior_tree.get_node_action(n);

        let root = behavior_tree.root();
        assert_eq!(
            behavior_tree.get_node_action(root),
            &NodeAction::LoopUntilAll
        );

        let mut neighbors = behavior_tree.get_node_neighbors(root);
        assert_eq!(
            neighbors.next().map(map_to_node_action),
            Some(&NodeAction::SetBoard {
                pattern: TRRBTPattern::from([(
                    "main",
                    toodee_pattern(
                        3,
                        3,
                        ["A", "B", "C", "D", "E", "F", "G", "H", "I"]
                    )
                ),])
            })
        );

        Ok(())
    }
}
