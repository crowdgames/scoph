use super::*;
use ::serde::{
    Deserializer,
    de::{DeserializeSeed, Error, Visitor},
};
use serde_json::Value;

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Graph, D::Error>
where
    D: Deserializer<'de>,
{
    struct NodeSeed<'g> {
        graph: &'g mut Graph,
        parent: Option<NodeIndex>,
    }

    impl<'g, 'de> DeserializeSeed<'de> for NodeSeed<'g> {
        type Value = ();
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let v = serde_json::Value::deserialize(deserializer)?;

            let info =
                Info::deserialize(&v).map_err(|_| Error::custom("Couldn't cheat with json"))?;
            let action = NodeAction::deserialize(&v)
                .map_err(|_| Error::custom("Couldn't cheat with json"))?;

            let ni = self.graph.add_node(action, info);
            if let Some(parent) = self.parent {
                self.graph.add_edge(parent, ni);
            }

            if let Some(Value::Array(children)) = v.get("children") {
                for child in children {
                    NodeSeed {
                        graph: self.graph,
                        parent: Some(ni),
                    }
                    .deserialize(child)
                    .map_err(|_| Error::custom("Couldn't cheat with json"))?;
                }
            }

            Ok(())
        }
    }

    let mut out = Default::default();
    NodeSeed {
        graph: &mut out,
        parent: None,
    }
    .deserialize(deserializer)?;
    Ok(out)
}
