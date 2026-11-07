use super::*;
use pretty_assertions::assert_eq;

#[test]
fn test_player_choice() -> Result<()> {
    const GAME: &str = r#"
    {"name":"Player Test","desc":"Player wins with first rewrite","tree":{"type":"order","children":[{"type":"set-board","comment":"","nid":"","pattern":{"main":[["X","Y"]]}},{"type":"win","comment":"","nid":"","children":[{"type":"player","comment":"","nid":"","children":[{"type":"x-ident","comment":"","nid":"x_to_zero","children":[{"type":"rewrite","comment":"","nid":"","button":"","lhs":{"main":[["X"]]},"rhs":{"main":[["0"]]}}]},{"type":"x-swap","comment":"","nid":"y_to_zero","children":[{"type":"x-link","comment":"","nid":"","target":"x_to_zero"}],"what":"X","with":"Y"}],"pid":"Jeremy"}],"pid":"Jeremy"}],"comment":"","nid":""}}
    "#;

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(GAME)?);

    let x_pat = TRRBTPattern::from([("main", [["X"]])]);
    let y_pat = TRRBTPattern::from([("main", [["Y"]])]);
    let zero_pat = TRRBTPattern::from([("main", [["0"]])]);

    let mut chs = Chooser {
        cmp: Comparer {
            expected_name: "Player Test".to_string(),
            expected_description: "Player wins with first rewrite".to_string(),
            board_states: vec![],
            current_stack: Default::default(),
        },
        expected_choices: [
            ((x_pat.clone(), zero_pat.clone()), [(0, 0)]),
            ((y_pat.clone(), zero_pat.clone()), [(1, 0)]),
        ]
        .into(),
    };

    // game will run until we must collect our choices
    intrp.run::<_, Bare>(&mut chs)?;
    intrp.player_choose(0, 0)?;
    intrp.resume::<_, Full>(&mut chs)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));
    assert_eq!(intrp.board, TRRBTPattern::from([("main", [["0", "Y"]])]));

    Ok(())
}

#[test]
fn test_player_no_rewrites() -> Result<()> {
    const GAME: &str = r#"
    {"name":"Player Test Fail (from template)","desc":"Player can't make any rewrites","tree":{"type":"order","children":[{"type":"set-board","comment":"","nid":"","pattern":{"main":[["Z","Z"]]}},{"type":"lose","comment":"","nid":"","children":[{"type":"none","comment":"","nid":"","children":[{"type":"player","comment":"","nid":"","children":[{"type":"x-ident","comment":"","nid":"x_to_zero","children":[{"type":"rewrite","comment":"","nid":"","button":"","lhs":{"main":[["X"]]},"rhs":{"main":[["0"]]}}]},{"type":"x-swap","comment":"","nid":"y_to_zero","children":[{"type":"x-link","comment":"","nid":"","target":"x_to_zero"}],"what":"X","with":"Y"}],"pid":"Jeremy"}]}],"pid":"mr_evil"}],"comment":"","nid":""}}
    "#;

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(GAME)?);

    let mut chs = Chooser {
        cmp: Comparer {
            expected_name: "Player Test Fail (from template)".to_string(),
            expected_description: "Player can't make any rewrites".to_string(),
            board_states: vec![],
            current_stack: Default::default(),
        },
        expected_choices: PlayerChoices::default(),
    };

    // game will run until we must collect our choices
    intrp.run::<_, Bare>(&mut chs)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Lose("mr_evil".into())));
    assert_eq!(intrp.board, TRRBTPattern::from([("main", [["Z", "Z"]])]));

    Ok(())
}

#[test]
fn test_tic_tac_toe_row_race() -> Result<()> {
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_file(
        "../games/trees/tic-tac-toe.json",
    )?);

    let us_pat = TRRBTPattern::from([("main", [["_"]])]);
    let x_pat = TRRBTPattern::from([("main", [["X"]])]);
    let o_pat = TRRBTPattern::from([("main", [["O"]])]);

    let mut chs = Chooser {
        cmp: Comparer {
            expected_name: "tic-tac-toe".to_string(),
            expected_description: String::new(),
            board_states: vec![],
            current_stack: Default::default(),
        },
        expected_choices: PlayerChoices::from([(
            (us_pat.clone(), x_pat.clone()),
            [
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2),
            ],
        )]),
    };

    intrp.run_full(&mut chs)?;
    intrp.player_choose(0, 0)?;

    chs.expected_choices = PlayerChoices::from([(
        (us_pat.clone(), o_pat.clone()),
        [
            (1, 0),
            (2, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ],
    )]);

    intrp.resume::<_, Full>(&mut chs)?;
    intrp.player_choose(0, 5)?;

    chs.expected_choices = PlayerChoices::from([(
        (us_pat.clone(), x_pat.clone()),
        [(1, 0), (2, 0), (0, 1), (1, 1), (2, 1), (1, 2), (2, 2)],
    )]);

    intrp.resume::<_, Full>(&mut chs)?;
    intrp.player_choose(0, 0)?;

    chs.expected_choices = PlayerChoices::from([(
        (us_pat.clone(), o_pat.clone()),
        [(2, 0), (0, 1), (1, 1), (2, 1), (1, 2), (2, 2)],
    )]);

    intrp.resume::<_, Full>(&mut chs)?;
    intrp.player_choose(0, 4)?;

    chs.expected_choices = PlayerChoices::from([(
        (us_pat.clone(), x_pat.clone()),
        [(2, 0), (0, 1), (1, 1), (2, 1), (2, 2)],
    )]);

    intrp.resume::<_, Full>(&mut chs)?;
    intrp.player_choose(0, 0)?;

    intrp.resume::<_, Full>(&mut chs)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("X".into())));

    Ok(())
}
