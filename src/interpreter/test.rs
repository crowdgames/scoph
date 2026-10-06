use super::*;
use crate::common::toodee_pattern;
use anyhow::Result;
use pretty_assertions::assert_eq;

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

    interpreter.run(&mut cmp)?;

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
    int.startup(&mut cmp);
    for _ in 0..500 {
        int.step(&mut cmp)?;
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

    int.run_with_stack_trace(&mut cmp)?;

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
