use toodee::Coordinate;

use super::*;
use anyhow::Result;
use pretty_assertions::assert_eq;

#[test]
fn test_pattern_matches_single_char_and_layer() {
    let board = TRRBTPattern::from([("main", toodee_pattern(6, 6, ["."; 36]))]);
    let matchee = TRRBTPattern::from([("main", toodee_pattern(1, 1, ["."]))]);

    let board_matches: Vec<_> = board.matches(&matchee).collect();

    let mut expected_coords: Vec<Coordinate> = Vec::new();
    for y in 0..6 {
        for x in 0..6 {
            expected_coords.push((x, y));
        }
    }

    for (layer_name, match_coords) in board_matches {
        assert_eq!(layer_name, "main");
        assert_eq!(match_coords.collect::<Vec<Coordinate>>(), expected_coords);
    }
}

#[test]
fn test_pattern_rewrite() -> Result<()> {
    let mut dest = TRRBTPattern::from([("main", toodee_pattern(6, 6, ["."; 36]))]);
    let rx = TRRBTPattern::from([("main", toodee_pattern(1, 1, ["X"]))]);
    dest.rewrite_at(0, 0, "main", &rx)?;

    let ro = TRRBTPattern::from([("main", toodee_pattern(2, 2, ["O"; 4]))]);
    dest.rewrite_at(1, 1, "main", &ro)?;

    let rb = TRRBTPattern::from([("main", toodee_pattern(3, 3, ["B"; 9]))]);
    dest.rewrite_at(3, 3, "main", &rb)?;

    assert_eq!(
        dest,
        TRRBTPattern::from([(
            "main",
            toodee_pattern(
                6,
                6,
                [
                    "X", ".", ".", ".", ".", ".", ".", "O", "O", ".", ".", ".", ".", "O", "O", ".",
                    ".", ".", ".", ".", ".", "B", "B", "B", ".", ".", ".", "B", "B", "B", ".", ".",
                    ".", "B", "B", "B",
                ]
            )
        )])
    );

    Ok(())
}

// transform tests
#[test]
fn test_rotated_90_tic_tac_toe() {
    let og = TRRBTPattern::from([("main", toodee_pattern(1, 3, ["X", "X", "X"]))]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", toodee_pattern(3, 1, ["X", "X", "X"]))])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_2() {
    let og = TRRBTPattern::from([("main", toodee_pattern(3, 1, ["X", "X", "X"]))]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", toodee_pattern(1, 3, ["X", "X", "X"]))])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_diag() {
    let og = TRRBTPattern::from([(
        "main",
        toodee_pattern(3, 3, ["X", ".", ".", ".", "X", ".", ".", ".", "X"]),
    )]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, [".", ".", "X", ".", "X", ".", "X", ".", "."])
        )])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_diag2() {
    let og = TRRBTPattern::from([(
        "main",
        toodee_pattern(3, 3, [".", ".", "X", ".", "X", ".", "X", ".", "."]),
    )]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, ["X", ".", ".", ".", "X", ".", ".", ".", "X"])
        )])
    );
}

#[test]
fn test_skew_tic_tac_toe() {
    let og = TRRBTPattern::from([("main", toodee_pattern(3, 1, ["X", "X", "X"]))]);
    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, ["X", ".", ".", ".", "X", ".", ".", ".", "X"])
        )])
    )
}

// basing these tests on what the web editor already does
#[test]
fn test_skew_perfectly_vertical() {
    let og = TRRBTPattern::from([("main", toodee_pattern(1, 3, ["X", "X", "X"]))]);
    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([("main", toodee_pattern(1, 3, ["X", "X", "X"]))])
    )
}

#[test]
fn test_skew_multirow() {
    let og = TRRBTPattern::from([(
        "main",
        toodee_pattern(3, 3, ["O", "X", "O", "O", "X", "O", "O", "X", "O"]),
    )]);

    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(
                3,
                5,
                [
                    "O", ".", ".", "O", "X", ".", "O", "X", "O", ".", "X", "O", ".", ".", "O"
                ]
            )
        )])
    );
}

#[test]
fn test_swap_only_on_pid_single_char() {
    let me = PlayerId(c"Jeremy".into());
    assert_eq!(me.swap_chars("e", "a"), "Jaramy".into());
}

#[test]
fn test_swap_only_on_pid_single_to_many_char() {
    let me = PlayerId(c"Jeremy".into());
    assert_eq!(me.swap_chars("e", "ooo"), "Jooorooomy".into());
}

#[test]
fn test_swap_only_on_existing() {
    let ex = PlayerId(c"XOXO".into());
    assert_eq!(ex.swap_chars("X", "O"), "OXOX".into());
}

#[test]
fn test_swap_only_on_pattern_single_char() {
    let og = TRRBTPattern::from([(
        "main",
        toodee_pattern(3, 3, ["O", "X", "O", "O", "X", "O", "O", "X", "O"]),
    )]);

    assert_eq!(
        og.swap_chars("O", "X"),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(3, 3, ["X", "O", "X", "X", "O", "X", "X", "O", "X"]),
        )])
    );
}

#[test]
fn test_swap_only_on_pattern_single_char_to_many_char() {
    let og = TRRBTPattern::from([(
        "main",
        toodee_pattern(3, 3, ["O", "X", "O", "O", "X", "O", "O", "X", "O"]),
    )]);

    assert_eq!(
        og.swap_chars("O", "OOO"),
        TRRBTPattern::from([(
            "main",
            toodee_pattern(
                3,
                3,
                ["OOO", "X", "OOO", "OOO", "X", "OOO", "OOO", "X", "OOO"]
            ),
        )])
    );
}
