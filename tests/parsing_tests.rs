use anyhow::{Result, anyhow};
use pretty_assertions::assert_eq;
use scoph::trrbt::*;

fn load_json_from_test(filename: &str) -> Result<TRRBTDocument> {
    let bytes = std::fs::read(&format!("games/trees/{filename}"))?;
    serde_json::from_slice(&bytes).map_err(|e| anyhow!("Failed to serialize: {e}"))
}

#[test]
fn test_parsing_loop() -> Result<()> {
    let doc = load_json_from_test("loop.json")?;
    assert_eq!(doc.name, "loop");

    let mut root = Node::new(NodeAction::Order);
    root.add_child(Node::new(NodeAction::SetBoard { pattern: Pattern::filled("_", 3, 1) }));
    let mut loop_until_all = Node::new(NodeAction::LoopUntilAll);
    let mut player = Node::new(NodeAction::Player { pid: PlayerId("1".to_string()) });

    let rewrite1 = Node::new(NodeAction::Rewrite { lhs: Pattern::filled("_", 1, 1), rhs: Pattern::filled("X", 1, 1) });
    player.add_child(rewrite1);
    let rewrite2 = Node::new(NodeAction::Rewrite { lhs: Pattern::filled("_", 1, 1), rhs: Pattern::filled("O", 1, 1) });
    player.add_child(rewrite2);

    let mut match_loop = Node::new(NodeAction::LoopUntilAll);
    match_loop.add_child(Node::new(NodeAction::Match { pattern: Pattern::filled("O", 1, 1) }));

    loop_until_all.add_child(player);
    loop_until_all.add_child(match_loop);

    root.add_child(loop_until_all);
    assert_eq!(doc.tree, root);

    Ok(())
}

#[test]
fn test_parsing_tic_tac_toe() -> Result<()> {
    let doc = load_json_from_test("tic-tac-toe.json")?;
    assert_eq!(doc.name, "tic-tac-toe");

    let mut root = Node::new(NodeAction::Order);

    let initial_board = Node::new(NodeAction::SetBoard {
        pattern: Pattern::filled("_", 3, 3),
    });
    root.add_child(initial_board);

    let mut loop_until_all = Node::new(NodeAction::LoopUntilAll);
    loop_until_all.set_id("gameloop");

    let mut x_ident = Node::new(NodeAction::XIdent);
    x_ident.set_id("ply");

    let mut player_node = Node::new(NodeAction::Player {
        pid: PlayerId("X".to_string()),
    });
    let player_rewrite = Node::new(NodeAction::Rewrite {
        lhs: Pattern::filled("_", 1, 1),
        rhs: Pattern::filled("X", 1, 1),
    });
    player_node.add_child(player_rewrite);
    x_ident.add_child(player_node);

    let mut win = Node::new(NodeAction::Win {
        pid: PlayerId("X".to_string()),
    });
    let mut player_x_rotate = Node::new(NodeAction::default_x_rotate());
    let mut player_x_skew = Node::new(NodeAction::default_x_skew());
    let initial_win_match = Node::new(NodeAction::Match {
        pattern: Pattern::filled("X", 3, 1),
    });

    player_x_skew.add_child(initial_win_match);
    player_x_rotate.add_child(player_x_skew);
    win.add_child(player_x_rotate);
    x_ident.add_child(win);

    let mut draw = Node::new(NodeAction::Draw);
    let mut none = Node::new(NodeAction::None);
    let draw_match = Node::new(NodeAction::Match {
        pattern: Pattern::filled("_", 1, 1),
    });
    none.add_child(draw_match);
    draw.add_child(none);
    x_ident.add_child(draw);

    let mut swap_chars = Node::new(NodeAction::XSwap {
        what: "X".to_string(),
        with: "O".to_string(),
    });
    let x_link = Node::new(NodeAction::XLink {
        target: "ply".to_string(),
    });
    swap_chars.add_child(x_link);

    loop_until_all.add_child(x_ident);
    loop_until_all.add_child(swap_chars);
    root.add_child(loop_until_all);

    assert_eq!(doc.tree, root);

    Ok(())
}
