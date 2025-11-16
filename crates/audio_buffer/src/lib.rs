pub use dasp;
pub use symphonia;

pub mod buffers;
pub mod core;

pub trait SharedSample: dasp::Sample + Send + Sync + 'static {}
impl<T> SharedSample for T where T: dasp::Sample + Send + Sync + 'static {}

#[cfg(feature = "loader")]
pub mod loader;
