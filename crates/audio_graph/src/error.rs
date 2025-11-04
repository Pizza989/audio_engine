use daggy::{NodeIndex, WouldCycle};
use thiserror::Error;

use crate::Connection;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node Index {0:?} would dangling")]
    WouldInvalidNode(NodeIndex),
    #[error("Connection {0} would cycle")]
    WouldCycle(#[from] WouldCycle<Connection>),
    #[error("")]
    WouldInvalidPinMatrix,
    #[error("")]
    WouldDanglingNodeInConnection,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("The supplied buffers didn't match the processors input or output configuration")]
    InvalidBuffers,
}
