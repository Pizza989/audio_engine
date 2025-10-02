use std::error::Error;

use crate::core::io::error::IoError;

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("file not found {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("unknown format {0}")]
    UnkownFormat(symphonia::core::errors::Error),
    #[error("no track found")]
    NoTrackFound,
    #[error("no channels found")]
    NoChannelsFound,
    #[error("unknown channel format {0}")]
    UnkownChannelFormat(usize),
    #[error("file too large {0}")]
    FileTooLarge(usize),
    #[error("could not create decoder {0}")]
    CouldNotCreateDecoder(symphonia::core::errors::Error),
    #[error("error while decoding {0}")]
    ErrorWhileDecoding(symphonia::core::errors::Error),
    #[error("unexpected error while decoding {0}")]
    UnexpectedErrorWhileDecoding(Box<dyn Error>),
    #[error("error converting buffers")]
    IoError(#[from] IoError),
}
