use petgraph::graph::NodeIndex;
use thiserror::Error;

use crate::common::NodeAction;

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
    MissingLayerOnSrc(String),

    #[error("Tree does not have a node with ID \"{0}\"")]
    MissingNodeId(String),

    #[error("Attempted to call startup after it was already called")]
    AlreadyStarted,

    #[error("Attempted to call step without having called startup")]
    HasntStarted,

    #[error("An x-link transform points to a target that doesn't exist")]
    BadLink { target: String },

    #[error("A player node has a child that is neither transform nor rewrite: ({0:?}) {2:#?}")]
    BadPlayerChild(NodeIndex, NodeAction),
}
