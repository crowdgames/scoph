use std::assert_matches;

use super::*;
use crate::common::toodee_pattern;
use anyhow::Result;
use pretty_assertions::assert_eq;
use serde_json::json;

struct Comparer {
    expected_name: String,
    expected_description: String,
    board_states: Vec<TRRBTPattern>,
}

impl Visitor for Comparer {
    fn startup(&mut self, tree_name: &str, tree_description: &str) {
        assert_eq!(self.expected_name, tree_name);
        assert_eq!(self.expected_description, tree_description);
    }

    fn view_board(&mut self, board: &TRRBTPattern) {
        self.board_states.push(board.clone());
    }
}

#[test]
fn test_only_set_board() -> Result<()> {
    const SINGLE_SET_BOARD: &str = r#"
    {
        "name" : "Only set-board",
        "desc" : "A single set-board node on the call stack",
        "tree": {
            "type": "set-board",
            "pattern": {
                "main": [
                    ["👁️‍🗨️", "_", "👁️‍🗨️"],
                    ["_", "👃", "_"],
                    ["<", "---", ">"]
                ]
            }
        }
    }
    "#;

    let bt = BehaviorTree::load_from_json_text(SINGLE_SET_BOARD)?;
    let mut interpreter = Interpreter::new(bt);
    let mut cmp = Comparer {
        expected_name: "Only set-board".to_string(),
        expected_description: "A single set-board node on the call stack".to_string(),
        board_states: vec![],
    };

    interpreter.run_full(&mut cmp)?;

    assert_eq!(
        cmp.board_states.as_slice(),
        &[TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, ["👁️‍🗨️", "_", "👁️‍🗨️", "_", "👃", "_", "<", "---", ">"])
        )])]
    );

    Ok(())
}

#[test]
fn test_loop_until_all_with_set_board() -> Result<()> {
    const LOOP_AND_SET: &str = r#"
    {
        "name" : "loop-until-all + set-board",
        "desc" : "Expected to run forever because set-board always succeeds",
        "tree": {
            "type" : "loop-until-all",
            "nid": "",
            "comment": "",
            "children" : [
                {
                    "type": "set-board",
                    "pattern": { "main": [ ["👁️‍🗨️", "_", "👁️‍🗨️"], ["_", "👃", "_"], ["<", "---", ">"] ]}
                }
            ]
        }
    }
    "#;

    let bt = BehaviorTree::load_from_json_text(LOOP_AND_SET)?;
    let mut int = Interpreter::new(bt);

    let mut cmp = Comparer {
        expected_name: "loop-until-all + set-board".to_string(),
        expected_description: "Expected to run forever because set-board always succeeds"
            .to_string(),
        board_states: vec![],
    };

    // Idea is that this tree will run forever, do to the semantics of the language
    int.startup(&mut cmp)?;
    for _ in 0..500 {
        assert_matches!(int.step(), Ok(Status::InProgress));
        int.view_board(&mut cmp);
    }

    let mut expected_vec = vec![
        TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, ["👁️‍🗨️", "_", "👁️‍🗨️", "_", "👃", "_", "<", "---", ">"])
        )]);
        499
    ];
    expected_vec.insert(0, TRRBTPattern::default());

    assert_eq!(cmp.board_states, expected_vec);

    Ok(())
}

#[test]
fn test_win_via_match() -> Result<()> {
    const GAME: &str = r#"
    {
        "name": "Single Match",
        "desc": "Game ends with single match",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [
                            [".", ".", "."],
                            [".", "X", "."],
                            [".", ".", "."]
                        ],
                        "other": [
                            ["|", "|", "|", "U"]
                        ]
                    }
                },
                {
                    "type": "win",
                    "pid": "Jeremy",
                    "children": [
                        {
                            "type": "match",
                            "pattern": {
                                "main": [["X"]],
                                "other": [["U"]]
                            }
                        }
                    ]
                }
            ]
        }
    }
    "#;

    let mut cmp = Comparer {
        expected_name: "Single Match".to_string(),
        expected_description: "Game ends with single match".to_string(),
        board_states: vec![],
    };
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(GAME)?);
    intrp.run_full(&mut cmp)?;
    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));

    Ok(())
}

#[test]
fn test_win_via_match_times() -> Result<()> {
    const GAME: &str = r#"{
        "name": "Win w/ Match Times",
        "desc": "Game ends if we match a pattern a certain number of times",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [
                            ["X", "X", "X"],
                            ["X", "X", "X"],
                            ["X", "X", "X"]
                        ],
                        "other": [
                            ["U", "U", "U"],
                            ["U", "U", "U"],
                            ["U", "U", "U"]
                        ]
                    }
                },
                {
                    "type": "win",
                    "pid": "Jeremy",
                    "children": [
                        {
                            "type": "match-times",
                            "times": 4,
                            "pattern": {
                                "main": [
                                    ["X", "X"],
                                    ["X", "X"]
                                ],
                               "other": [
                                    ["U", "U"],
                                    ["U", "U"]
                                ]
                            }
                        }
                    ]
                }
            ]
        }
    }"#;

    let mut cmp = Comparer {
        expected_name: "Win w/ Match Times".to_string(),
        expected_description: "Game ends if we match a pattern a certain number of times"
            .to_string(),
        board_states: vec![],
    };

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(GAME)?);
    intrp.run_full(&mut cmp)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));

    Ok(())
}

#[test]
fn test_loop_that_should_end() -> Result<()> {
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

    let expected_start_pattern: TRRBTPattern =
        TRRBTPattern::from([("main", toodee_pattern(6, 6, ["."; 36]))]);

    let expected_final_pattern: TRRBTPattern =
        TRRBTPattern::from([("main", toodee_pattern(6, 6, ["X"; 36]))]);

    let bt = BehaviorTree::load_from_json_text(ORDER_REPLACE_LOOP)?;
    let mut int = Interpreter::new(bt);

    let mut cmp = Comparer {
        expected_name:"Order and replace".to_string(),
        expected_description: "Run an order node with two children. First a set-board, then a loop-until-all with a single rewrite child".to_string(),
        board_states: vec![],
    };

    int.run_full(&mut cmp)?;

    let mut boards = cmp.board_states.into_iter();
    // initial step where we just push the set board
    assert_eq!(boards.next(), Some(TRRBTPattern::default()));
    // set board does its job
    assert_eq!(boards.next(), Some(expected_start_pattern.clone()));
    // order pushes loop-until-all, board's still the same
    assert_eq!(boards.next(), Some(expected_start_pattern.clone()));
    // loop-until-all pushes first rewrite, stays the same
    assert_eq!(boards.next(), Some(expected_start_pattern));

    let single_x = TRRBTPattern::from([("main", toodee_pattern(1, 1, ["X"]))]);
    let mut expected_x = 1;
    let mut push_frame = false;
    for _ in 0..(36usize * 2usize) {
        let board = boards.next().expect(&format!(
            "Should have replaced {} '.'s with 'X's already",
            expected_x
        ));

        for (layer_name, coords) in board.matches(&single_x) {
            assert_eq!(layer_name, "main");
            assert_eq!(coords.count(), expected_x);
        }

        if push_frame {
            expected_x += 1;
        }
        push_frame = !push_frame;
    }
    assert_eq!(boards.next(), Some(expected_final_pattern));

    Ok(())
}

fn make_rewrites(i: i32, len: i32) -> serde_json::Value {
    use serde_json::Value as V;
    let child_rewrites = (0..len)
        .into_iter()
        .map(|n| {
            json!({
                "type": "rewrite",
                "lhs": {
                    "main": [ [if n >= i {"Y"} else {"N"}] ],
                },
                "rhs": {
                    "main": [[ "_" ]],
                }
            })
        })
        .collect();
    V::Array(child_rewrites)
}

#[test]
fn test_none_quits_on_success() -> Result<()> {
    const REWRITES: i32 = 5;
    for i in 0..REWRITES {
        let test_game = json!({
            "name": "None Quits",
            "desc": "A none node that will quit when it has a successful rewrite",
            "tree": {
                "type": "order",
                "children": [
                    {
                        "type": "set-board",
                        "pattern": {
                            "main": [["_", "_", "Y", "_", "_"]],
                        }
                    },
                    {
                        "type": "none",
                        "children": make_rewrites(i, REWRITES)
                    }
                ]
            }
        });

        let mut cmp = Comparer {
            expected_name: "None Quits".to_string(),
            expected_description: "A none node that will quit when it has a successful rewrite"
                .to_string(),
            board_states: vec![],
        };

        let mut intrp =
            Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);
        intrp.run_full(&mut cmp)?;

        let initial_pattern: TRRBTPattern =
            serde_json::from_value(json!({"main": [["_", "_", "Y", "_", "_"]]}))?;

        let mut v = vec![
            // start and push set-board
            TRRBTPattern::default(),
            // do and pop set-board
            initial_pattern.clone(),
            // push none
            initial_pattern.clone(),
        ];

        for _ in 0..=(i * 2) {
            v.push(initial_pattern.clone());
        }

        for _ in 0..3 {
            v.push(serde_json::from_value(
                json!({"main": [["_", "_", "_", "_", "_"]]}),
            )?);
        }

        assert_eq!(cmp.board_states, v, "at i: {i}");
    }
    Ok(())
}

#[test]
fn test_all_quits_on_failure() -> Result<()> {
    const REWRITES: i32 = 5;
    let underscore: TRRBTPattern = serde_json::from_value(json!({"main": [["_"]]}))?;
    for i in 0..REWRITES {
        let initial_pattern_value = json!({
            "main": [(0..REWRITES).map(|n| {
                if n < i {
                    "Y"
                } else {
                    "X"
                }
            }).collect::<Vec<&str>>()],
        });
        let initial_pattern: TRRBTPattern = serde_json::from_value(initial_pattern_value.clone())?;
        let test_game = json!({
            "name": "All Quits",
            "desc": "All quits after failing its first rewrite",
            "tree": {
                "type": "order",
                "children": [
                    {
                        "type": "set-board",
                        "pattern": initial_pattern_value
                    },
                    {
                        "type": "all",
                        "children": make_rewrites(-1, REWRITES)
                    }
                ]
            }
        });

        let mut cmp = Comparer {
            expected_name: "All Quits".to_string(),
            expected_description: "All quits after failing its first rewrite".to_string(),
            board_states: vec![],
        };

        let mut intrp =
            Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);
        intrp.run_full(&mut cmp)?;

        let mut iter = cmp.board_states.into_iter();
        // push set board
        assert_eq!(iter.next(), Some(TRRBTPattern::default()));
        // do set-board and pop
        assert_eq!(iter.next(), Some(initial_pattern.clone()));
        // push all
        assert_eq!(iter.next(), Some(initial_pattern.clone()));
        // push first rewrite
        assert_eq!(iter.next(), Some(initial_pattern.clone()));

        for j in 0..i {
            let expected_underscores = j + 1;
            let bs = iter.next().unwrap();
            assert_eq!(
                bs.matches(&underscore).next().unwrap().1.count(),
                expected_underscores as usize,
                "Didn't get the expected rewrite: {:?}",
                bs
            );
            let bs = iter.next().unwrap();
            assert_eq!(
                bs.matches(&underscore).next().unwrap().1.count(),
                expected_underscores as usize,
                "Didn't get the expected rewrite: {:?}",
                bs
            );
        }

        let final_pattern: TRRBTPattern = serde_json::from_value(json!({
            "main": [(0..REWRITES).map(|n| if n < i {
                "_"
            } else {
                "X"
            }).collect::<Vec<_>>()]
        }))?;
        while let Some(bs) = iter.next() {
            assert_eq!(bs, final_pattern);
        }
    }
    Ok(())
}

#[test]
fn test_none_finishes() -> Result<()> {
    const REWRITES: i32 = 5;
    let test_game = json!({
        "name": "None doesn't quit",
        "desc": "A none node that won't quit because all rewrites fail",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [["_", "_", "Y", "_", "_"]],
                    }
                },
                {
                    "type": "none",
                    "children": make_rewrites(REWRITES, REWRITES)
                }
            ]
        }
    });

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);
    let mut cmp = Comparer {
        expected_name: "None doesn't quit".to_string(),
        expected_description: "A none node that won't quit because all rewrites fail".to_string(),
        board_states: vec![],
    };
    intrp.run_full(&mut cmp)?;

    let initial_pattern: TRRBTPattern =
        serde_json::from_value(json!({"main": [["_", "_", "Y", "_", "_"]]}))?;
    let mut v = vec![
        TRRBTPattern::default(),
        initial_pattern.clone(),
        initial_pattern.clone(),
        initial_pattern.clone(),
    ];
    for _ in 0..=(REWRITES * 2) {
        v.push(initial_pattern.clone());
    }
    assert_eq!(cmp.board_states, v);

    Ok(())
}

#[test]
fn test_win_ends_game() -> Result<()> {
    const REWRITES: i32 = 5;
    let test_game = json!( {
        "name": "Win Test",
        "desc": "Win node will end its game upon its first success",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [["_", "_", "Y", "_", "_"]]
                    }
                },
                {
                    "type": "win",
                    "pid": "Jeremy",
                    "children": make_rewrites(REWRITES-1, REWRITES)
                }
            ]
        }
    });

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);

    let mut cmp = Comparer {
        expected_name: "Win Test".to_string(),
        expected_description: "Win node will end its game upon its first success".to_string(),
        board_states: vec![],
    };
    intrp.run_full(&mut cmp)?;

    assert_eq!(intrp.outcome, Some(Outcome::Win("Jeremy".into())));

    let mut v = vec![
        TRRBTPattern::default(),
        // set-board and pop
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push win
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push first rewrite
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
    ];

    for _ in 0..((REWRITES - 1) * 2) {
        v.push(serde_json::from_value(
            json!({ "main": [["_", "_", "Y", "_", "_"]] }),
        )?);
    }

    v.push(serde_json::from_value(
        json!({ "main": [["_", "_", "_", "_", "_"]]}),
    )?);
    assert_eq!(cmp.board_states, v);

    Ok(())
}

#[test]
fn test_lose_ends_game() -> Result<()> {
    const REWRITES: i32 = 5;
    let test_game = json!( {
        "name": "Lose Test",
        "desc": "Lose node will end its game upon its first success",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [["_", "_", "Y", "_", "_"]]
                    }
                },
                {
                    "type": "lose",
                    "pid": "Jeremy",
                    "children": make_rewrites(REWRITES-1, REWRITES)
                }
            ]
        }
    });

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);

    let mut cmp = Comparer {
        expected_name: "Lose Test".to_string(),
        expected_description: "Lose node will end its game upon its first success".to_string(),
        board_states: vec![],
    };
    intrp.run_full(&mut cmp)?;

    assert_eq!(intrp.outcome, Some(Outcome::Lose("Jeremy".into())));

    let mut v = vec![
        TRRBTPattern::default(),
        // set-board and pop
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push win
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push first rewrite
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
    ];

    for _ in 0..((REWRITES - 1) * 2) {
        v.push(serde_json::from_value(
            json!({ "main": [["_", "_", "Y", "_", "_"]] }),
        )?);
    }

    v.push(serde_json::from_value(
        json!({ "main": [["_", "_", "_", "_", "_"]]}),
    )?);
    assert_eq!(cmp.board_states, v);

    Ok(())
}

#[test]
fn test_tie_ends_game() -> Result<()> {
    const REWRITES: i32 = 5;
    let test_game = json!( {
        "name": "Lose Test",
        "desc": "Lose node will end its game upon its first success",
        "tree": {
            "type": "order",
            "children": [
                {
                    "type": "set-board",
                    "pattern": {
                        "main": [["_", "_", "Y", "_", "_"]]
                    }
                },
                {
                    "type": "draw",
                    "children": make_rewrites(REWRITES-1, REWRITES)
                }
            ]
        }
    });

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(&test_game.to_string())?);

    let mut cmp = Comparer {
        expected_name: "Lose Test".to_string(),
        expected_description: "Lose node will end its game upon its first success".to_string(),
        board_states: vec![],
    };
    intrp.run_full(&mut cmp)?;

    assert_eq!(intrp.outcome, Some(Outcome::Tie));

    let mut v = vec![
        TRRBTPattern::default(),
        // set-board and pop
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push win
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
        // push first rewrite
        serde_json::from_value(json!({ "main": [["_", "_", "Y", "_", "_"]] }))?,
    ];

    for _ in 0..((REWRITES - 1) * 2) {
        v.push(serde_json::from_value(
            json!({ "main": [["_", "_", "Y", "_", "_"]] }),
        )?);
    }

    v.push(serde_json::from_value(
        json!({ "main": [["_", "_", "_", "_", "_"]]}),
    )?);
    assert_eq!(cmp.board_states, v);

    Ok(())
}
