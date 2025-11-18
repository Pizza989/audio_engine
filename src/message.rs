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
    InvalidConnection {
        source: NodeIndex,
        destination: NodeIndex,
        matrix: PinMatrix,
    },
    Ok,
}

#[derive(Debug, Clone)]
pub struct AudioBackendMessage {
    pub id: MessageId,
    pub intent: Intent,
}

#[derive(Debug, Clone)]
pub enum Intent {
    Query(AudioBackendQuery),
    Command(AudioBackendCommand),
}

#[derive(Debug, Clone)]
pub enum AudioBackendQuery {}

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
