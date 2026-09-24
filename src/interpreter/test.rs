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
