use audio_graph::{
    daggy::{EdgeIndex, NodeIndex},
    pin_matrix::PinMatrix,
};
use time::MusicalTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);

#[derive(Debug, Clone)]
pub struct AudioEngineMessage {
    pub id: MessageId,
    pub status: AudioEngineStatus,
}

#[derive(Debug, Clone)]
pub enum AudioEngineStatus {
    AddNode(NodeIndex),
    RemoveNode(NodeIndex),
    AddEdge {
        index: EdgeIndex,
        source: NodeIndex,
        destination: NodeIndex,
    },
    RemoveEdge(EdgeIndex),
}

#[derive(Debug, Clone)]
pub struct AudioBackendMessage {
    pub id: MessageId,
    pub command: AudioBackendCommand,
}

#[derive(Debug, Clone)]
pub enum AudioBackendCommand {
    Start,
    Pause,
    SetPlayhead(MusicalTime),
    AddTrack,
    AddConnection {
        source: NodeIndex,
        destination: NodeIndex,
        matrix: PinMatrix,
    },
    UpdateConnection {
        edge: EdgeIndex,
        matrix: PinMatrix,
    },
}
