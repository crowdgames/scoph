use toodee::Coordinate;

use super::*;
use pretty_assertions::assert_eq;
use anyhow::Result;

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

    assert_eq!(dest, TRRBTPattern::from([("main", toodee_pattern(6, 6, [
        "X", ".", ".", ".", ".", ".",
        ".", "O", "O", ".", ".", ".",
        ".", "O", "O", ".", ".", ".",
        ".", ".", ".", "B", "B", "B",
        ".", ".", ".", "B", "B", "B",
        ".", ".", ".", "B", "B", "B",
    ]))]));

    Ok(())
}
