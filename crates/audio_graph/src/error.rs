use daggy::{NodeIndex, WouldCycle};
use thiserror::Error;

use crate::Connection;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node Index {0:?} was dangling")]
    InvalidNode(NodeIndex),
    #[error("Connection {0} would cycle")]
    WouldCycle(#[from] WouldCycle<Connection>),
    #[error(
        "Connections must fulfill the invariant src.output_channels == dst.input_channels: {0} != {1}"
    )]
    InvalidConnection(usize, usize),
    #[error("output and input nodes must always be valid")]
    OutputInputValidity,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("The supplied buffers didn't match the processors input or output configuration")]
    InvalidBuffers,
}
