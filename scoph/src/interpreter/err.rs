use petgraph::graph::NodeIndex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IError {
    #[error("Missing Child at {child}, node has {child_count} children")]
    MissingChild {
        node: NodeIndex,
        child: u32,
        child_count: u32,
    },
    #[error("Destination pattern does not have a layer named {0}")]
    MissingLayerOnDest(String),
    #[error("Source pattern does not have a layer named {0}")]
    MissingLayerOnSrc(String)
}
