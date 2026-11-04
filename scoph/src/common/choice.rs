use ordermap::OrderMap;
use toodee::Coordinate;

use super::TRRBTPattern;

pub type Rewrite = (TRRBTPattern, TRRBTPattern);
pub type MatchCoords = Vec<Coordinate>;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct PlayerChoices(OrderMap<Rewrite, MatchCoords>);

impl PlayerChoices {
    pub fn insert(&mut self, rw: Rewrite, coords: MatchCoords) {
        if !coords.is_empty() {
            self.0.insert(rw, coords);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_index(&self, index: usize) -> Option<(&Rewrite, &[Coordinate])> {
        self.0
            .get_index(index)
            .map(|(rw, coord)| (rw, coord.as_slice()))
    }

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
