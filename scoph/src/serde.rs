use std::marker::PhantomData;

use crate::TRRBTTree;
use crate::common::Info;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer};
use toodee::TooDee;

pub fn deserialize_trrbt_tree<'de, D>(deserializer: D) -> Result<TRRBTTree, D::Error>
where
    D: Deserializer<'de>,
{
    struct TreeWalker<'a>(&'a mut TRRBTTree, Option<NodeIndex>);
    impl<'de> DeserializeSeed<'de> for TreeWalker {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de> {
            // get node info first
            let 
        }
    }


    struct TRRBTTreeVisitor;

    impl<'de> Visitor<'de> for TRRBTTreeVisitor {
        type Value = TRRBTTree;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "A recursive graph of nodes")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut out = Self::Value::new_undirected();

            Ok(out)
        }
    }

    deserializer.deserialize_map(TRRBTTreeVisitor)
}

pub fn deserialize_2d_array<'de, T, D>(deserializer: D) -> Result<TooDee<T>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::de::Deserializer<'de>,
{
    struct TooDeeRow<'a, T: 'a>(&'a mut Vec<T>);
    impl<'de, 'a, T> DeserializeSeed<'de> for TooDeeRow<'a, T>
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
