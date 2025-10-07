use crate::core::axis::{BufferAxis, BufferAxisMut};

pub mod axis;
pub mod io;
pub mod stride;

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
    type FrameMut<'this>: BufferAxisMut<'this, Self::Sample>
    where
        Self: 'this;
    type ChannelMut<'this>: BufferAxisMut<'this, Self::Sample>
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

    // SAFETY: this is safe because
    // 1. a mutable frame is not returned
    // 2. the passed closure can not return the frame either
    //
    // This does not compile!
    // ```rust
    //    let mut buffer = FixedFrameBuffer {
    //      data: vec![[0.; 256]],
    //      sample_rate: 44_100,
    //    };
    //    let mut view = buffer.with_frame_mut(0, |view| view).unwrap();
    // ```
    fn with_frame_mut<'s, F, R>(&'s mut self, index: usize, f: F) -> Option<R>
    where
        F: for<'this> FnOnce(Self::FrameMut<'this>) -> R,
        R: 's;
    fn with_channel_mut<'s, F, R>(&'s mut self, index: usize, f: F) -> Option<R>
    where
        F: for<'this> FnOnce(Self::ChannelMut<'this>) -> R,
        R: 's;
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
