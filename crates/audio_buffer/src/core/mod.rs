use crate::core::buffer_axis::{BufferAxis, BufferAxisMut};

pub mod buffer_axis;
pub mod io;
pub mod stride;

/// A general abstraction over multi-channel, resizeable audio buffers.
///
/// - Storage layout (frame-major, channel-major, interleaved, etc.)
///   is determined by the implementor.
pub trait Buffer {
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
    fn channels(&self) -> usize;
    fn samples(&self) -> usize;
    fn sample_rate(&self) -> usize;
}

pub trait BufferMut: Buffer {
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

    fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'this>) -> R;
    fn with_channel_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'this>) -> R;
}

pub trait ResizableBuffer: Buffer {
    /// Resize the buffer, truncating data if shrinking
    fn resize(&mut self, frames: usize);

    /// Grow the buffer to at least the specified size.
    /// This must never shrink the buffer.
    fn ensure_capacity(&mut self, min_frames: usize) {
        if self.frames() < min_frames {
            self.resize(min_frames);
        }
    }

    /// Truncate the buffer to the specified size.
    /// This must never grow the buffer.
    fn truncate(&mut self, max_frames: usize) {
        if self.frames() > max_frames {
            self.resize(max_frames);
        }
    }
}
