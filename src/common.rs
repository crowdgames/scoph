use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};
use toodee::TooDee;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub String);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pattern {
    // this just pulls straight from the JSON. Can be optimized,
    // but we shall kick that can
    #[serde(deserialize_with = "deserialize_2d_array")]
    main: TooDee<String>,
}

impl Pattern {
    // mainly for testing
    pub fn filled(cell: &str, width: usize, height: usize) -> Self {
        Self {
            main: TooDee::init(width, height, cell.to_string()),
        }
    }
}

fn deserialize_2d_array<'de, T, D>(deserializer: D) -> Result<TooDee<T>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Deserializer;
    use serde::de::Visitor;

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
        pattern: Pattern,
    },
    AppendRows {
        pattern: Pattern,
    },
    AppendColumns {
        pattern: Pattern,
    },
    Rewrite {
        lhs: Pattern,
        rhs: Pattern,
    },
    RewriteAll {
        lhs: Pattern,
        rhs: Pattern,
    },
    LayerTemplate {
        layer: String,
        with: String,
    },

    // Condition Nodes,
    Match {
        pattern: Pattern,
    },
    MatchTimes {
        #[serde(default = "one_times")]
        times: u32,
        pattern: Pattern,
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
