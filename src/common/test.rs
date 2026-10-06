use toodee::Coordinate;

use super::*;
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
