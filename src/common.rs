use std::{collections::HashMap, marker::PhantomData, ops::Deref};

use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, Deserializer, Visitor},
};
use serde_with::{NoneAsEmptyString, serde_as};
use toodee::{CopyOps, TooDee, TooDeeOps, TooDeeOpsMut, TooDeeViewMut};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub String);

impl From<&str> for PlayerId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

type LayersAndPatterns = HashMap<String, TooDee<String>>;

pub fn toodee_pattern<'a, const N: usize>(
    width: usize,
    height: usize,
    flat_cells: [&'a str; N],
) -> TooDee<String> {
    let v: Vec<_> = flat_cells.iter().map(ToString::to_string).collect();
    TooDee::from_vec(width, height, v)
}

/// Error enum signifying that there was no layer with the given name in either the destination
/// pattern or from the source pattern
#[derive(Debug)]
pub enum MissingLayer {
    Dest(String),
    Src(String),
}

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct TRRBTPattern(LayersAndPatterns);

impl TRRBTPattern {
    pub fn filled(cell: &str, width: usize, height: usize) -> Self {
        let mut core = LayersAndPatterns::new();
        core.insert(
            "main".to_string(),
            TooDee::init(width, height, cell.to_string()),
        );
        Self(core)
    }

    /// Returns an iterator of the layer names and an iterator of their rows on the board
    pub fn layers(&self) -> impl Iterator<Item = (&str, impl Iterator<Item = &[String]>)> {
        self.0
            .iter()
            .map(|(layer_name, board)| (layer_name.as_str(), board.rows()))
    }

    /// Gets the size of patterns with their associated layers
    pub fn pattern_sizes(&self) -> impl Iterator<Item = (&str, (usize, usize))> {
        self.0
            .iter()
            .map(|(layer_name, data)| (layer_name.as_str(), data.size()))
    }

    /// Returns an iterator of coordinates where the provided pattern was matched top-left
    pub fn matches(
        &self,
        other: &Self,
    ) -> impl Iterator<Item = (&str, impl Iterator<Item = (usize, usize)>)> {
        self.0.iter().filter_map(|(layer_name, data)| {
            let other_data = other
                .0
                .get(layer_name)
                .expect("Patterns must have the same layers to be matched");

            let (other_width, other_height) = other_data.size();

            let mut coords = Vec::new();
            let (mut x, mut y) = (0usize, 0usize);
            loop {
                if x + other_width > data.num_cols() {
                    if y + other_height > data.num_rows() {
                        break;
                    }
                    x = 0;
                    y += 1;
                }

                let view = data.view((x, y), (x + other_width, y + other_height));
                if view
                    .cells()
                    .zip(other_data.cells())
                    .all(|(board_view, pattern)| board_view == pattern)
                {
                    coords.push((x, y));
                }

                x += 1;
            }

            if coords.is_empty() {
                None
            } else {
                Some((layer_name.as_str(), coords.into_iter()))
            }
        })
    }

    /// Writes the pattern to to board at the given layer and position. Returns an error if the
    /// layer is not found in the destination or source patterns.
    pub fn rewrite_at(
        &mut self,
        x: usize,
        y: usize,
        layer_name: &str,
        src_pattern: &Self,
    ) -> Result<(), MissingLayer> {
        let src_data = src_pattern
            .0
            .get(layer_name)
            .ok_or(MissingLayer::Src(layer_name.to_string()))?;

        let (src_width, src_height) = src_data.size();
        self.0
            .get_mut(layer_name)
            .ok_or(MissingLayer::Dest(layer_name.to_string()))?
            .view_mut((x, y), (src_width, src_height))
            .clone_from_toodee(src_data);

        Ok(())
    }
}

impl<'a, const N: usize> From<[(&'a str, TooDee<String>); N]> for TRRBTPattern {
    fn from(value: [(&'a str, TooDee<String>); N]) -> Self {
        let mut core = LayersAndPatterns::new();
        for (key, value) in value {
            core.insert(key.to_string(), value);
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
            type Value = TooDee<String>;

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

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

                Ok(TRRBTPattern(layers_and_patterns))
            }
        }

        deserializer.deserialize_map(TRRBTPatternVisitor)
    }
}

fn deserialize_2d_array<'de, T, D>(deserializer: D) -> Result<TooDee<T>, D::Error>
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

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[serde_as]
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Info {
    #[serde_as(as = NoneAsEmptyString)]
    #[serde(default)]
    nid: Option<String>,
    #[serde_as(as = NoneAsEmptyString)]
    #[serde(default)]
    comment: Option<String>,
}

impl Info {
    pub fn set_id(&mut self, id: &str) {
        self.nid = Some(id.to_string());
    }
}

fn one_times() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum NodeAction {
    // Tree Macros
    XIdent,
    XPrune,
    XMirror {
        #[serde(default)]
        remorig: bool,
    },
    XSkew {
        #[serde(default)]
        remorig: bool,
    },
    XRotate {
        #[serde(default)]
        remorig: bool,
    },
    XSpin {
        #[serde(default)]
        remorig: bool,
    },
    XFlip {
        #[serde(default)]
        remorig: bool,
    },
    XSwap {
        what: String,
        with: String,
    },
    XReplace {
        what: String,
        withs: Vec<String>,
    },
    XUnrollReplace {
        what: String,
    },
    XLink {
        target: String,
    },

    // Player & Win Conditions
    Player {
        pid: PlayerId,
    },
    Win {
        pid: PlayerId,
    },
    Lose {
        pid: PlayerId,
    },
    Draw,

    // control flow nodes
    Order,
    All,
    None,
    RandomTry,
    LoopUntilAll,
    LoopTimes {
        #[serde(default = "one_times")]
        times: u32,
    },

    // Rewrite nodes,
    SetBoard {
        pattern: TRRBTPattern,
    },
    AppendRows {
        pattern: TRRBTPattern,
    },
    AppendColumns {
        pattern: TRRBTPattern,
    },
    Rewrite {
        lhs: TRRBTPattern,
        rhs: TRRBTPattern,
    },
    RewriteAll {
        lhs: TRRBTPattern,
        rhs: TRRBTPattern,
    },
    LayerTemplate {
        layer: String,
        with: String,
    },

    // Condition Nodes,
    Match {
        pattern: TRRBTPattern,
    },
    MatchTimes {
        #[serde(default = "one_times")]
        times: u32,
        pattern: TRRBTPattern,
    },
}

impl NodeAction {
    pub fn default_x_rotate() -> Self {
        Self::XRotate { remorig: false }
    }

    pub fn default_x_skew() -> Self {
        Self::XSkew { remorig: false }
    }
}
