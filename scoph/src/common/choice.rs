use crate::prelude::*;
use ordermap::OrderMap;
use toodee::Coordinate;

use super::TRRBTPattern;

/// A 2-tuple of patterns. The left-hand side will be searched for on the board, and the right-hand
/// side is the aforementioned's replacement.
pub type Rewrite = (TRRBTPattern, TRRBTPattern);
/// A list of coordinates where the left-hand side of a rewrite was matched and can be rewritten.
pub type MatchCoords = Vec<Coordinate>;

/// An ordered map of [Rewrite]'s to lists of coordinates where they matched on the board.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct PlayerChoices(OrderMap<Rewrite, MatchCoords, fnv::FnvBuildHasher>);

impl PlayerChoices {
    /// Inserts a rewrite and list of coordinates. Will not be inserted if coords is empty.
    pub fn insert(&mut self, rw: Rewrite, coords: MatchCoords) {
        if !coords.is_empty() && !self.0.contains_key(&rw) {
            self.0.insert(rw, coords);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the rewrite and a slice of its matched coordinates if it exists.
    /// Indexes are respective to the transformed tree.
    pub fn get_index(&self, index: usize) -> Option<(&Rewrite, &[Coordinate])> {
        self.0
            .get_index(index)
            .map(|(rw, coord)| (rw, coord.as_slice()))
    }

    /// The count of rewrites able to be done on the board
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Rewrite, &[Coordinate])> {
        (0..self.len()).map(|i| self.get_index(i).unwrap())
    }
}

impl IntoIterator for PlayerChoices {
    type Item = (Rewrite, MatchCoords);

    type IntoIter = ordermap::map::IntoIter<Rewrite, MatchCoords>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, const O: usize> From<[(Rewrite, [Coordinate; O]); N]> for PlayerChoices {
    fn from(value: [(Rewrite, [Coordinate; O]); N]) -> Self {
        Self(
            value
                .into_iter()
                .map(|(rw, coords)| (rw, coords.into_iter().collect()))
                .collect(),
        )
    }
}
