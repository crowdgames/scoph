use std::path::Path;

use crate::{
    common::{Info, NodeAction},
    interpreter::IError,
    parse::{DocNode, TRRBTDocument},
};
use anyhow::anyhow;
use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug)]
pub struct BehaviorNode {
    info: Info,
    action: NodeAction,
}

#[derive(Debug, Default)]
pub struct BehaviorEdge;

type TRRBTTree = DiGraph<BehaviorNode, BehaviorEdge>;

// TODO: implement a custom serializer for BehaviorTree

/// A BehaviorTree is a non-mutating structure that represents the structure of a full TRRBT
/// program. They can be loaded from a TRRBT json file and be transformed.
#[derive(Debug)]
pub struct BehaviorTree {
    name: String,
    desc: String,
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
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Gets the description of the game
    pub fn description(&self) -> &str {
        self.desc.as_str()
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
    use pretty_assertions::assert_eq;

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
                    toodee_pattern(3, 3, ["A", "B", "C", "D", "E", "F", "G", "H", "I"])
                ),])
            })
        );

        Ok(())
    }

    #[test]
    fn test_complex_finishable_tree_from_text() -> Result<()> {
        const ORDER_REPLACE_LOOP: &str = r#"
    {
        "name": "Order and replace",
        "desc": "Run an order node with two children. First a set-board, then a loop-until-all with a single rewrite child",
        "tree": {
            "type": "order",
            "nid": "",
            "comment": "",
            "children": [
                {
                    "type": "set-board",
                    "nid": "",
                    "comment": "",
                    "pattern": {
                        "main": [
                            [".", ".", ".", ".", ".", "."],
                            [".", ".", ".", ".", ".", "."],
                            [".", ".", ".", ".", ".", "."],
                            [".", ".", ".", ".", ".", "."],
                            [".", ".", ".", ".", ".", "."],
                            [".", ".", ".", ".", ".", "."]
                        ]
                    }
                },
                {
                    "type": "loop-until-all",
                    "nid": "",
                    "comment": "",
                    "children": [
                        {
                            "type": "rewrite",
                            "nid": "",
                            "comment": "",
                            "lhs": {
                                "main": [
                                    ["."]
                                ]
                            },
                            "rhs": {
                                "main": [
                                    ["X"]
                                ]
                            }
                        }
                    ]
                }
            ]
        }
    }
    "#;

        let behavior_tree = BehaviorTree::load_from_json_text(ORDER_REPLACE_LOOP)?;
        let root = behavior_tree.root();
        assert_eq!(behavior_tree.get_node_action(root), &NodeAction::Order);

        let root_children: Vec<_> = behavior_tree.get_node_neighbors(root).collect();
        let expected_root_children_actions = vec![
            NodeAction::SetBoard {
                pattern: TRRBTPattern::from([("main", toodee_pattern(6, 6, ["."; 36]))]),
            },
            NodeAction::LoopUntilAll,
        ];
        let mut i = 0;
        for ni in root_children {
            let na = behavior_tree.get_node_action(ni);
            assert_eq!(na, &expected_root_children_actions[i]);

            match na {
                &NodeAction::SetBoard { .. } => {
                    assert_eq!(behavior_tree.get_node_neighbors(ni).count(), 0);
                }
                &NodeAction::LoopUntilAll => {
                    let mut neighbors = behavior_tree.get_node_neighbors(ni);
                    assert_eq!(
                        neighbors.next().map(|ni| behavior_tree.get_node_action(ni)),
                        Some(&NodeAction::Rewrite {
                            lhs: TRRBTPattern::from([("main", toodee_pattern(1, 1, ["."]))]),
                            rhs: TRRBTPattern::from([("main", toodee_pattern(1, 1, ["X"]))])
                        })
                    );
                    assert_eq!(neighbors.next(), None);
                }
                _ => unreachable!(),
            }
            i += 1;
        }

        Ok(())
    }

    #[test]
    fn test_multiple_children() -> Result<()> {
        const MULTIPLE_CHILDREN: &str = r#"{
            "name": "multiple children",
            "desc": "multiple children under an order node",
            "tree": {
                "type": "order",
                "children": [
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["A"]]
                        }
                    },
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["B"]]
                        }
                    },
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["C"]]
                        }
                    },
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["D"]]
                        }
                    },
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["E"]]
                        }
                    }
                ]
            }
        }"#;

        let behavior_tree = BehaviorTree::load_from_json_text(MULTIPLE_CHILDREN)?;
        assert_eq!(behavior_tree.name(), "multiple children");
        assert_eq!(
            behavior_tree.description(),
            "multiple children under an order node"
        );
        let root = behavior_tree.root();
        assert_eq!(behavior_tree.get_node_action(root), &NodeAction::Order);

        for (ni, pattern_text) in behavior_tree
            .get_node_neighbors(root)
            .zip(["A", "B", "C", "D", "E"])
        {
            assert_eq!(
                behavior_tree.get_node_action(ni),
                &NodeAction::SetBoard {
                    pattern: TRRBTPattern::from([("main", toodee_pattern(1, 1, [pattern_text]))])
                }
            )
        }
        Ok(())
    }
}
