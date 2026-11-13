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
    assert_eq!(behavior_tree.name(), c"A simple game");
    assert_eq!(behavior_tree.description(), c"From a simpler time");

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
        assert_eq!(na, &expected_root_children_actions[i], "{:#?}", behavior_tree);

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
                    }),
                    "{:#?}",
                    behavior_tree
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
    assert_eq!(behavior_tree.name(), c"multiple children");
    assert_eq!(
        behavior_tree.description(),
        c"multiple children under an order node"
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


use crate::prelude::*;
