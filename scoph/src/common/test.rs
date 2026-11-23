use toodee::Coordinate;

use super::*;
use crate::prelude::*;
use anyhow::Result;
use pretty_assertions::assert_eq;

#[test]
fn test_pattern_matches_single_char_and_layer() {
    let board = pattern::TRRBTPattern::from([("main", [["."; 6]; 6])]);
    let matchee = pattern::TRRBTPattern::from([("main", [["."]])]);

    let board_matches: Vec<_> = board.matches(&matchee).collect();

    let mut expected_coords: Vec<Coordinate> = Vec::new();
    for y in 0..6 {
        for x in 0..6 {
            expected_coords.push((x, y));
        }
    }

    assert_eq!(board_matches, expected_coords);
}

#[test]
fn test_pattern_rewrite() -> Result<()> {
    let mut dest = pattern::TRRBTPattern::from([("main", [["."; 6]; 6])]);
    let rx = pattern::TRRBTPattern::from([("main", [["X"]])]);
    dest.rewrite_at(0, 0, &rx)?;

    let ro = pattern::TRRBTPattern::from([("main", [["O"; 2]; 2])]);
    dest.rewrite_at(1, 1, &ro)?;

    let rb = pattern::TRRBTPattern::from([("main", [["B"; 3]; 3])]);
    dest.rewrite_at(3, 3, &rb)?;

    assert_eq!(
        dest,
        TRRBTPattern::from([(
            "main",
            [
                ["X", ".", ".", ".", ".", "."],
                [".", "O", "O", ".", ".", "."],
                [".", "O", "O", ".", ".", "."],
                [".", ".", ".", "B", "B", "B"],
                [".", ".", ".", "B", "B", "B"],
                [".", ".", ".", "B", "B", "B"],
            ]
        )])
    );

    Ok(())
}

// transform tests
#[test]
fn test_rotated_90_tic_tac_toe() {
    let og = pattern::TRRBTPattern::from([("main", [["X", "X", "X"]])]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", [["X"], ["X"], ["X"]])])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_2() {
    let og = TRRBTPattern::from([("main", [["X", "X", "X"]])]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", [["X"], ["X"], ["X"]])])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_diag() {
    let og = TRRBTPattern::from([("main", [["X", ".", "."], [".", "X", "."], [".", ".", "X"]])]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", [[".", ".", "X"], [".", "X", "."], ["X", ".", "."]])])
    );
}

#[test]
fn test_rotate_90_tic_tac_toe_diag2() {
    let og = TRRBTPattern::from([("main", [[".", ".", "X"], [".", "X", "."], ["X", ".", "."]])]);

    assert_eq!(
        og.rotated_90(),
        TRRBTPattern::from([("main", [["X", ".", "."], [".", "X", "."], [".", ".", "X"]])])
    );
}

#[test]
fn test_skew_tic_tac_toe() {
    let og = TRRBTPattern::from([("main", [["X", "X", "X"]])]);
    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([("main", [["X", ".", "."], [".", "X", "."], [".", ".", "X"]])])
    )
}

// basing these tests on what the web editor already does
#[test]
fn test_skew_perfectly_vertical() {
    let og = TRRBTPattern::from([("main", [["X"], ["X"], ["X"]])]);
    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([("main", [["X"], ["X"], ["X"]])])
    )
}

#[test]
fn test_skew_multirow() {
    let og = TRRBTPattern::from([("main", [["O", "X", "O"], ["O", "X", "O"], ["O", "X", "O"]])]);

    assert_eq!(
        og.skewed(),
        TRRBTPattern::from([(
            "main",
            [
                ["O", ".", "."],
                ["O", "X", "."],
                ["O", "X", "O"],
                [".", "X", "O"],
                [".", ".", "O"]
            ]
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
    let og = TRRBTPattern::from([("main", [["O", "X", "O"], ["O", "X", "O"], ["O", "X", "O"]])]);

    assert_eq!(
        og.swap_chars("O", "X"),
        TRRBTPattern::from([("main", [["X", "O", "X"], ["X", "O", "X"], ["X", "O", "X"]],)])
    );
}

#[test]
fn test_swap_only_on_pattern_single_char_to_many_char() {
    let og = TRRBTPattern::from([("main", [["O", "X", "O"], ["O", "X", "O"], ["O", "X", "O"]])]);

    assert_eq!(
        og.swap_chars("O", "OOO"),
        TRRBTPattern::from([(
            "main",
            [
                ["OOO", "X", "OOO"],
                ["OOO", "X", "OOO"],
                ["OOO", "X", "OOO"]
            ]
        )])
    );
}

#[test]
fn test_append_rows_too_small() -> Result<()> {
    let mut pat = TRRBTPattern::from([(
        "main",
        [
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
        ],
    )]);

    let smaller_pat = TRRBTPattern::from([("main", [["O", "Y"], ["Q", "Y"]])]);
    pat.append_rows(&smaller_pat)?;
    assert_eq!(
        pat,
        TRRBTPattern::from([(
            "main",
            [
                ["X", "X", "X", "X"],
                ["X", "X", "X", "X"],
                ["X", "X", "X", "X"],
                ["O", "Y", "O", "Y"],
                ["Q", "Y", "Q", "Y"],
            ],
        )])
    );

    Ok(())
}

#[test]
fn test_append_rows_too_big() -> Result<()> {
    let mut pat = TRRBTPattern::from([(
        "main",
        [
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
        ],
    )]);

    let smaller_pat = TRRBTPattern::from([(
        "main",
        [
            ["O", "O", "O", "O", "O", "O"],
            ["O", "O", "O", "O", "O", "O"],
            ["O", "O", "O", "O", "O", "O"],
            ["O", "O", "O", "O", "O", "O"],
        ],
    )]);
    pat.append_rows(&smaller_pat)?;
    assert_eq!(
        pat,
        TRRBTPattern::from([(
            "main",
            [
                ["X", "X", "X", "X"],
                ["X", "X", "X", "X"],
                ["X", "X", "X", "X"],
                ["O", "O", "O", "O"],
                ["O", "O", "O", "O"],
                ["O", "O", "O", "O"],
                ["O", "O", "O", "O"],
            ],
        )])
    );

    Ok(())
}

#[test]
fn test_append_columns_too_small() -> Result<()> {
    let mut pat = TRRBTPattern::from([(
        "main",
        [
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
        ],
    )]);

    let smaller_pat = TRRBTPattern::from([("main", [["O", "Y"], ["Q", "Y"]])]);

    pat.append_columns(&smaller_pat)?;

    assert_eq!(
        pat,
        TRRBTPattern::from([(
            "main",
            [
                ["X", "X", "X", "X", "O", "Y"],
                ["X", "X", "X", "X", "Q", "Y"],
                ["X", "X", "X", "X", "O", "Y"],
                ["X", "X", "X", "X", "Q", "Y"],
            ]
        )])
    );

    Ok(())
}

#[test]
fn test_append_colmuns_too_big() -> Result<()> {
    let mut pat = TRRBTPattern::from([(
        "main",
        [
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
            ["X", "X", "X", "X"],
        ],
    )]);
    let bigger_pat = TRRBTPattern::from([(
        "main",
        [
            ["O", "Y"],
            ["Q", "Y"],
            ["B", "B"],
            ["B", "B"],
            ["B", "B"],
            ["B", "B"],
        ],
    )]);

    pat.append_columns(&bigger_pat)?;

    assert_eq!(
        pat,
        TRRBTPattern::from([(
            "main",
            [
                ["X", "X", "X", "X", "O", "Y"],
                ["X", "X", "X", "X", "Q", "Y"],
                ["X", "X", "X", "X", "B", "B"],
                ["X", "X", "X", "X", "B", "B"],
            ],
        )])
    );

    Ok(())
}

#[test]
fn test_row_mirror_not_centered() {
    let pat = TRRBTPattern::from([("main", [["x", "x"], ["y", "y"]])]);
    assert_eq!(
        pat.row_mirror(false),
        TRRBTPattern::from([("main", [["x", "x"], ["y", "y"], ["y", "y"], ["x", "x"],])])
    );
}

#[test]
fn test_row_mirror_centered() {
    let pat = TRRBTPattern::from([("main", [["x", "x"], ["y", "y"]])]);
    assert_eq!(
        pat.row_mirror(true),
        TRRBTPattern::from([("main", [["x", "x"], ["y", "y"], ["x", "x"],])])
    );
}

#[test]
fn test_col_mirror_not_centered() {
    let pat = TRRBTPattern::from([("main", [["x", "y"], ["x", "y"]])]);

    assert_eq!(
        pat.col_mirror(false),
        TRRBTPattern::from([("main", [["x", "y", "y", "x"], ["x", "y", "y", "x"],])])
    );
}

#[test]
fn test_col_mirror_centered() {
    let pat = TRRBTPattern::from([("main", [["x", "y"], ["x", "y"]])]);

    assert_eq!(
        pat.col_mirror(true),
        TRRBTPattern::from([("main", [["x", "y", "x"], ["x", "y", "x"],])])
    );
}
