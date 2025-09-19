use crate::core::buffer_axis::{BufferAxis, BufferAxisMut};

pub mod buffer_axis;
pub mod io;
pub mod stride;

/// A general abstraction over multi-channel, resizeable audio buffers.
///
/// - The number of channels is fixed at compile time (`const C`).
/// - Storage layout (frame-major, channel-major, interleaved, etc.)
///   is determined by the implementor.
pub trait Buffer<const C: usize> {
    type Sample: dasp::Sample;

    type Frame<'this>: BufferAxis<Self::Sample>
    where
        Self: 'this;
    type Channel<'this>: BufferAxis<Self::Sample>
    where
        Self: 'this;

    type IterFrames<'this>: Iterator<Item = Self::Frame<'this>>
    where
        Self: 'this;
    type IterChannels<'this>: Iterator<Item = Self::Channel<'this>>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>>;
    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>>;

    fn iter_frames(&self) -> Self::IterFrames<'_>;
    fn iter_channels(&self) -> Self::IterChannels<'_>;

    fn frames(&self) -> usize {
        self.samples() / self.channels()
    }
    fn channels(&self) -> usize {
        C
    }
    fn samples(&self) -> usize;
}

pub trait BufferMut<const C: usize>: Buffer<C> {
    type FrameMut<'this>: BufferAxisMut<Self::Sample>
    where
        Self: 'this;
    type ChannelMut<'this>: BufferAxisMut<Self::Sample>
    where
        Self: 'this;

    type IterFramesMut<'this>: Iterator<Item = Self::FrameMut<'this>>
    where
        Self: 'this;
    type IterChannelsMut<'this>: Iterator<Item = Self::ChannelMut<'this>>
    where
        Self: 'this;

    fn iter_frames_mut(&mut self) -> Self::IterFramesMut<'_>;
    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_>;

    fn with_frame_mut<F, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'_>) -> R;
    fn with_channel_mut<F, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'_>) -> R;
}
