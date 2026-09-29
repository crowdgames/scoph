use crate::common::toodee_pattern;
use pretty_assertions::assert_eq;
use super::*;
use anyhow::Result;

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
