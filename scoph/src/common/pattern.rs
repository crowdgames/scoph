use alloc::collections::BTreeMap;
use core::marker::PhantomData;
use core::str::FromStr;

use crate::prelude::*;
use itertools::Itertools;
use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, Deserializer, Visitor},
};

use toodee::{Coordinate, CopyOps, TooDee, TooDeeOps, TooDeeOpsMut};

use crate::common::swap_chars;
use crate::interpreter::IError;

use alloc::ffi::CString;

type LayersAndPatterns = BTreeMap<String, TooDee<CString>>;

pub fn toodee_pattern<const N: usize>(
    width: usize,
    height: usize,
    flat_cells: [&str; N],
) -> TooDee<String> {
    let v: Vec<_> = flat_cells.iter().map(ToString::to_string).collect();
    TooDee::from_vec(width, height, v)
}

/// A key-value structure of layer names ([CString]s) and 2D arrays
#[derive(Debug, Default, Hash, Clone, Serialize, PartialEq, Eq)]
pub struct TRRBTPattern(LayersAndPatterns);

impl TRRBTPattern {
    /// Creates a pattern with a single layer named "main," filled with the provided default cell at
    /// the specified dimensions.
    pub fn filled(cell: &str, width: usize, height: usize) -> Self {
        let mut core = LayersAndPatterns::new();
        core.insert(
            "main".to_string(),
            TooDee::init(
                width,
                height,
                CString::from_str(cell).expect("Regular string slice"),
            ),
        );
        Self(core)
    }

    /// Returns an iterator of the layer names and an iterator of their rows on the board
    pub fn layers(
        &self,
    ) -> impl Iterator<Item = (&str, impl Iterator<Item = impl Iterator<Item = &str>>)> {
        self.0.iter().map(|(layer_name, board)| {
            (
                layer_name.as_str(),
                board
                    .rows()
                    .map(|s| s.iter().map(|cs| cs.to_str().unwrap())),
            )
        })
    }

    /// Gets the size of patterns with their associated layers
    pub fn pattern_size(&self) -> (usize, usize) {
        self.0
            .iter()
            .next()
            .map(|(_, data)| data.size())
            .unwrap_or((0, 0))
    }

    /// Returns an iterator of coordinates where the provided pattern was matched. All matches are
    /// anchored to the top-left of the pattern.
    pub fn matches(&self, other: &Self) -> impl Iterator<Item = Coordinate> {
        let (self_width, self_height) = self.pattern_size();
        let (other_width, other_height) = other.pattern_size();
        assert!(
            other_width <= self_width,
            "comparand pattern should fit within the dimensions of the comparer. other.width: {other_width} > self.width: {self_width}"
        );
        assert!(
            other_height <= self_height,
            "comparand pattern should fit within the dimensions of the comparer. other.height: {other_height} > self.height: {self_height}"
        );

        // NOTE: work w/ Anand to find a faster search
        let mut coords = vec![];
        for y in 0..=(self_height - other_height) {
            for x in 0..=(self_width - other_width) {
                if other.0.iter().all(|(layer_name, other_data)| {
                    self.0.get(layer_name).is_some_and(|self_data| {
                        self_data
                            .view((x, y), (x + other_width, y + other_height))
                            .cells()
                            .eq(other_data.cells())
                    })
                }) {
                    coords.push((x, y));
                }
            }
        }

        coords.into_iter()
    }

    /// Writes the pattern to to board at the given position. Returns an error if the
    /// patterns do not have the same layers.
    pub fn rewrite_at(&mut self, x: usize, y: usize, other: &Self) -> Result<(), IError> {
        let (sw, sh) = self.pattern_size();
        let (ow, oh) = other.pattern_size();
        assert!(sw >= ow);
        assert!(sh >= oh);

        for (layer_name, layer_data) in self.0.iter_mut() {
            let other_data = other
                .0
                .get(layer_name)
                .ok_or_else(|| IError::MissingLayerOnSrc(layer_name.to_owned()))?;
            layer_data
                .view_mut((x, y), (x + ow, y + oh))
                .clone_from_toodee(other_data);
        }

        Ok(())
    }

    /// Appends the columns of the other pattern to the caller. Will truncate the height of the
    /// other pattern if needed.
    pub fn append_columns(&mut self, other: &Self) -> Result<(), IError> {
        let (_, h) = self.pattern_size();
        for (layer_name, layer_data) in self.0.iter_mut() {
            let other_data = other
                .0
                .get(layer_name)
                .ok_or_else(|| IError::MissingLayerOnSrc(layer_name.clone()))?;

            for i in 0..other_data.num_cols() {
                let mut row_index = 0;
                let cool = other_data
                    .col(i)
                    .take(h)
                    .pad_using(h, |_| {
                        let cell = &other_data[(i, row_index)];
                        row_index = (row_index % other_data.num_rows()) + 1;
                        cell
                    })
                    .cloned();
                layer_data.push_col(cool);
            }
        }

        Ok(())
    }

    /// Appends the rows of the other pattern to the caller. Will truncate the width of the rows on
    /// the other pattern if needed.
    pub fn append_rows(&mut self, other: &Self) -> Result<(), IError> {
        let (w, _) = self.pattern_size();
        for (layer_name, layer_data) in self.0.iter_mut() {
            let other_data = other
                .0
                .get(layer_name)
                .ok_or_else(|| IError::MissingLayerOnSrc(layer_name.clone()))?;

            for i in 0..other_data.num_rows() {
                let mut col_index = 0;
                layer_data.push_row(
                    other_data[i]
                        .iter()
                        .take(w)
                        .pad_using(w, |_| {
                            let cell = &other_data[(col_index, i)];
                            col_index = (col_index % other_data.num_cols()) + 1;
                            cell
                        })
                        .cloned(),
                );
            }
        }

        Ok(())
    }

    /// Accesses a cell on the grid of the specified layer.
    pub fn get_cell(&self, x: usize, y: usize, layer_name: &str) -> Result<&str, IError> {
        self.0
            .get(layer_name)
            .ok_or(IError::MissingLayerOnSrc(layer_name.to_string()))
            .map(|grid| grid[(x, y)].to_str().unwrap())
    }

    /// Accesses a cell on the grid as a C string, meaning it will be null terminated.
    pub fn get_cell_c(
        &self,
        x: usize,
        y: usize,
        layer_name: &str,
    ) -> Result<*const core::ffi::c_char, IError> {
        self.0
            .get(layer_name)
            .ok_or(IError::MissingLayerOnSrc(layer_name.to_string()))
            .map(|grid| grid[(x, y)].as_ptr())
    }

    /// Creates a rotated version of this pattern 90 degrees clockwise.
    pub fn rotated_90(&self) -> Self {
        let mut out = LayersAndPatterns::new();

        for (layer_name, data) in self.0.iter() {
            let mut new_data = TooDee::with_capacity(data.num_cols() * data.num_rows());

            for col_index in 0..data.num_cols() {
                new_data.push_row(data.col(col_index).rev().map(ToOwned::to_owned));
            }

            out.insert(layer_name.to_owned(), new_data);
        }

        Self(out)
    }

    /// Creates a skewed version of this pattern, shifting rows into a diagonal line from the top
    /// left to bottom right.
    pub fn skewed(&self) -> Self {
        // 100% copied from the original web editor, translated to rust
        let mut out = LayersAndPatterns::new();

        for (layer_name, data) in self.0.iter() {
            let rows = data.num_rows();
            let cols = data.num_cols();
            let mut new_data = TooDee::with_capacity(rows * cols);

            for rr in 0..(rows + cols - 1) {
                new_data.push_row((0..cols).map(|cc| {
                    let skewed = rr.checked_sub(cc);
                    if let Some(skewed) = skewed
                        && skewed < rows
                    {
                        data[(cc, skewed)].clone()
                    } else {
                        c".".to_owned()
                    }
                }));
            }

            out.insert(layer_name.to_owned(), new_data);
        }

        Self(out)
    }

    /// Replaces all occurrences of [what] w/ [with] and vice-versa.
    pub fn swap_chars(&self, what: &str, with: &str) -> Self {
        Self(
            self.0
                .iter()
                .map(|(layer_name, layer_data)| {
                    let (w, h) = layer_data.size();
                    (
                        layer_name.clone(),
                        TooDee::from_vec(
                            w,
                            h,
                            layer_data
                                .into_iter()
                                .map(|cell| {
                                    CString::from_str(&swap_chars(
                                        cell.to_str().unwrap(),
                                        what,
                                        with,
                                    ))
                                    .expect("Replaced into valid UTF-8")
                                })
                                .collect(),
                        ),
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn set_layer(
        &mut self,
        layer: &str,
        width: usize,
        height: usize,
        data: Vec<CString>,
    ) {
        self.0
            .entry(layer.to_string())
            .and_modify(|layer_data| layer_data.clone_from_slice(&data))
            .or_insert(TooDee::from_vec(width, height, data));
    }
}

impl ToString for TRRBTPattern {
    fn to_string(&self) -> String {
        let mut out = String::new();

        for (k, v) in self.0.iter() {
            let mut ln = format!("{:?}", k.to_string());
            ln.push('\n');

            for r in v.rows() {
                for c in r {
                    ln.push_str(c.to_str().unwrap());
                    ln.push(' ');
                }
                ln.push('\n');
            }
            out.push_str(ln.as_str());
        }

        out
    }
}

impl<'a, const N: usize> From<[(&'a str, TooDee<String>); N]> for TRRBTPattern {
    fn from(value: [(&'a str, TooDee<String>); N]) -> Self {
        let mut core = LayersAndPatterns::new();
        for (key, value) in value {
            let w = value.num_cols();
            let h = value.num_rows();
            core.insert(
                key.to_string(),
                TooDee::from_vec(
                    w,
                    h,
                    value
                        .into_iter()
                        .map(|s| CString::from_str(s.as_str()).unwrap())
                        .collect(),
                ),
            );
        }
        assert!(
            core.values().map(|d| d.size()).all_equal(),
            "pattern layers must all be the same width and height"
        );
        Self(core)
    }
}

impl<'a, const N: usize, const O: usize, const P: usize> From<[(&'a str, [[&'a str; P]; O]); N]>
    for TRRBTPattern
{
    fn from(value: [(&'a str, [[&'a str; P]; O]); N]) -> Self {
        let mut core = LayersAndPatterns::new();
        for (layer_name, arr) in value {
            let mut layer_data = TooDee::with_capacity(O * P);
            for row in arr {
                layer_data.push_row(row.into_iter().map(|s| CString::from_str(s).unwrap()));
            }
            core.insert(layer_name.to_string(), layer_data);
        }
        Self(core)
    }
}

impl<'de> Deserialize<'de> for TRRBTPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PatternText;

        impl<'de> DeserializeSeed<'de> for PatternText {
            type Value = TooDee<CString>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserialize_2d_array(deserializer)
            }
        }

        struct TRRBTPatternVisitor;

        impl<'de> Visitor<'de> for TRRBTPatternVisitor {
            type Value = TRRBTPattern;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "A map of 2D arrays")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut layers_and_patterns = LayersAndPatterns::new();
                while let Some(layer_name) = map.next_key()? {
                    layers_and_patterns.insert(layer_name, map.next_value_seed(PatternText)?);
                }

                if layers_and_patterns.values().map(|data| data.size())
                    .all_equal()
                {
                    Ok(TRRBTPattern(layers_and_patterns))
                } else {
                    Err(serde::de::Error::custom("Patterns must be the same size"))
                }
            }
        }

        deserializer.deserialize_map(TRRBTPatternVisitor)
    }
}

pub(crate) fn deserialize_2d_array<'de, T, D>(deserializer: D) -> Result<TooDee<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct TooDeeRow<'a, T: 'a>(&'a mut Vec<T>);
    impl<'de, 'a, T> serde::de::DeserializeSeed<'de> for TooDeeRow<'a, T>
    where
        T: Deserialize<'de>,
    {
        // tells how many elements were in this row for validation
        type Value = usize;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct TooDeeRowVisitor<'a, T: 'a>(&'a mut Vec<T>);

            impl<'de, 'a, T> Visitor<'de> for TooDeeRowVisitor<'a, T>
            where
                T: Deserialize<'de>,
            {
                type Value = usize;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(formatter, "an array of strings")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    if let Some(size_hint) = seq.size_hint() {
                        self.0.reserve(size_hint);
                    }

                    let mut row_items = 0;
                    while let Some(elem) = seq.next_element()? {
                        self.0.push(elem);
                        row_items += 1;
                    }

                    Ok(row_items)
                }
            }

            deserializer.deserialize_seq(TooDeeRowVisitor(self.0))
        }
    }

    struct TooDeeVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for TooDeeVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = TooDee<T>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("A 2D Array with columns of even length")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut inner_vec = Vec::new();

            let mut prev_width = 0;
            let mut rows_counted = 0;
            while let Some(row_width) = seq.next_element_seed(TooDeeRow(&mut inner_vec))? {
                if prev_width == 0 {
                    prev_width = row_width;
                }

                if prev_width != row_width {
                    return Err(serde::de::Error::custom(
                        "2D Array does not have consistent width",
                    ));
                }

                rows_counted += 1;
            }

            Ok(TooDee::from_vec(prev_width, rows_counted, inner_vec))
        }
    }

    let visitor = TooDeeVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}

