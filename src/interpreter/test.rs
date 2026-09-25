use crate::common::toodee_pattern;

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

    interpreter.run(&mut cmp);

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
    use std::thread;
    use std::time::{Duration, Instant};
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

    struct ThreadComparer(Comparer);
    impl Visitor for ThreadComparer {
        fn startup(&mut self, tree_name: &str, tree_description: &str) {
            Comparer::startup(&mut self.0, tree_name, tree_description);
        }

        fn view_board(&mut self, board: &TRRBTPattern) {
            assert_eq!(
                board,
                &TRRBTPattern::from([(
                    "main",
                    toodee_pattern(3, 3, ["👁️‍🗨️", "_", "👁️‍🗨️", "_", "👃", "_", "<", "---", ">"])
                )])
            );
        }

        fn stack_trace(&mut self, tree: &BehaviorTree, call_stack: &[Call]) {
            Comparer::stack_trace(&mut self.0, tree, call_stack);
        }
    }

    let mut cmp = ThreadComparer(Comparer {
        expected_name: "loop-until-all + set-board".to_string(),
        expected_description: "Expected to run forever because set-board always succeeds"
            .to_string(),
        board_states: vec![],
    });

    let t = thread::spawn(move || {
        int.run(&mut cmp);
    });

    let start = Instant::now();
    loop {
        if Instant::now() - start > Duration::from_secs(3) {
            println!("Went on for the expected 3 seconds");
            // thread leak, don't do this in actual production
            drop(t);
            break;
        }
    }

    Ok(())
}
