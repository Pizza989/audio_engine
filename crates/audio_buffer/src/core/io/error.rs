#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("channel numbers didn't match {0} != {1}")]
    ChannelMismatch(usize, usize),
}
