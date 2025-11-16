use audio_graph::{daggy::NodeIndex, pin_matrix::PinMatrix};
use time::MusicalTime;

#[derive(Debug)]
pub struct AudioEngineMessage {
    pub id: MessageId,
    pub status: AudioEngineStatus,
}

#[derive(Debug)]
pub enum AudioEngineStatus {
    InvalidConnection {
        source: NodeIndex,
        destination: NodeIndex,
        matrix: PinMatrix,
    },
}

#[derive(Debug)]
pub struct AudioBackendMessage {
    pub id: MessageId,
    pub command: AudioBackendCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(u64);

#[derive(Debug)]
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
}
