// TODO: Current deserializer allocates a lot of junk we want removed. I'd like to have a
// deserializer that just takes from the serialized text directly, but it's a lot of work for an
// optimization we may not need yet. Keeping this in the repo for later.
use super::BehaviorTree;
use super::TRRBTTree;

use petgraph::graph::NodeIndex;
use serde::de::Error;
use serde::de::{DeserializeSeed, Visitor};
use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for BehaviorTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "kebab-case")]
        enum Field {
            Name,
            Desc,
            Tree,
        }

        struct TreeSeed<'a> {
            graph: &'a mut TRRBTTree,
            root_index: Option<NodeIndex>,
        }

        impl<'a, 'de> DeserializeSeed<'de> for TreeSeed<'a> {
            type Value = NodeIndex;

            fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct ChildVisitor<'a> { graph: &'a mut TRRBTTree, root: &'a mut Option<NodeIndex>, parent: Option<NodeIndex> }

                impl<'a, 'de> Visitor<'de> for ChildVisitor<'a> {
                    type Value = ();
                    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        
                        Ok(())
                    }
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "Expecting a valid node action, and potential nid's or comments")
                    }
                }

                deserializer.deserialize_map(ChildVisitor { graph: self.graph, root: &mut self.root_index, parent: None })?;

                self.root_index.ok_or_else(|| Error::custom("Tree has no children"))
            }
        }

        struct BehaviorTreeVisitor;

        impl<'de> Visitor<'de> for BehaviorTreeVisitor {
            type Value = BehaviorTree;

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let (mut name, mut desc, mut tree, mut root) = (None, None, None, None);

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Desc => {
                            if desc.is_some() {
                                return Err(Error::duplicate_field("desc"));
                            }
                            desc = Some(map.next_value()?);
                        }
                        Field::Tree => {
                            if tree.is_some() {
                                return Err(Error::duplicate_field("tree"));
                            }
                            let mut core = TRRBTTree::new();
                            let ri = map.next_value_seed(TreeSeed {
                                graph: &mut core,
                                root_index: None,
                            })?;
                            root = Some(ri);
                            tree = Some(core);
                        }
                    }
                }

                Ok(BehaviorTree {
                    name: name.ok_or_else(|| Error::missing_field("name"))?,
                    desc: desc.ok_or_else(|| Error::missing_field("desc"))?,
                    tree: tree.ok_or_else(|| Error::missing_field("tree"))?,
                    root_index: root.unwrap(),
                })
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "Expected a map with the fields 'name', 'desc', and 'tree'"
                )
            }
        }

        deserializer.deserialize_map(BehaviorTreeVisitor)
    }
}
