use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum LoadError {
    FileNotFound(std::io::Error),
    UnkownFormat(symphonia::core::errors::Error),
    NoTrackFound,
    NoChannelsFound,
    UnkownChannelFormat(usize),
    FileTooLarge(usize),
    CouldNotCreateDecoder(symphonia::core::errors::Error),
    ErrorWhileDecoding(symphonia::core::errors::Error),
    UnexpectedErrorWhileDecoding(Box<dyn Error>),
}

impl Error for LoadError {}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoadError::*;

        match self {
            FileNotFound(e) => write!(f, "File not found: {}", e),
            UnkownFormat(e) => write!(f, "Format not supported: {}", e),
            NoTrackFound => write!(f, "No default track found"),
            NoChannelsFound => write!(f, "No channels found"),
            UnkownChannelFormat(num_channels) => {
                write!(f, "Unkown channel format: {} channels found", num_channels)
            }
            FileTooLarge(max_bytes) => {
                write!(f, "File is too large: maximum is {} bytes", max_bytes)
            }
            CouldNotCreateDecoder(e) => write!(f, "Failed to create decoder: {}", e),
            ErrorWhileDecoding(e) => write!(f, "Error while decoding: {}", e),
            UnexpectedErrorWhileDecoding(e) => write!(f, "Unexpected error while decoding: {}", e),
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        Self::FileNotFound(e)
    }
}
