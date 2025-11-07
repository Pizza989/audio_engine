use std::{sync::Arc, time::Duration};

use time::SampleRate;

use crate::core::axis::{BufferAxis, BufferAxisMut};

pub mod axis;
pub mod io;

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
    fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.frames() as f64 / self.sample_rate())
    }
    fn channels(&self) -> usize;
    fn samples(&self) -> usize;
    fn sample_rate(&self) -> SampleRate;
}

pub trait BufferMut: Buffer {
    type FrameMut<'this>: BufferAxisMut<'this, Self::Sample>
    where
        Self: 'this;
    type ChannelMut<'this>: BufferAxisMut<'this, Self::Sample>
    where
        Self: 'this;

    /// Takes a closure that gets an exclusive borrow to a frame passed to it.
    /// Returning `None` indicates that the index was out of bounds.
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

    /// Apply a function to all frames in the buffer mutably. Returning
    /// `None` from this function instantly breaks out of the mapping loop.
    fn map_frames_mut<F, R>(&mut self, mut f: F, offset: Option<usize>)
    where
        F: for<'frame> FnMut(Self::FrameMut<'frame>, usize) -> Option<R>,
    {
        let num_frames = self.frames();
        for index in offset.unwrap_or(0)..num_frames {
            if let None = self
                .with_frame_mut(index, |frame| f(frame, index))
                .expect("index is never out of bounds")
            {
                break;
            };
        }
    }

    fn map_channels_mut<F, R>(&mut self, mut f: F, offset: Option<usize>)
    where
        F: for<'channel> FnMut(Self::ChannelMut<'channel>, usize) -> Option<R>,
    {
        let num_channels = self.channels();
        for index in offset.unwrap_or(0)..num_channels {
            if let None = self
                .with_channel_mut(index, |frame| f(frame, index))
                .expect("index is never out of bounds")
            {
                break;
            };
        }
    }

    fn set_to_equilibrium(&mut self);
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

impl<T> Buffer for Arc<T>
where
    T: Buffer,
{
    type Sample = T::Sample;

    type Frame<'this>
        = T::Frame<'this>
    where
        Self: 'this;

    type Channel<'this>
        = T::Channel<'this>
    where
        Self: 'this;

    type IterFrames<'this>
        = T::IterFrames<'this>
    where
        Self: 'this;

    type IterChannels<'this>
        = T::IterChannels<'this>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        (**self).get_frame(index)
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        (**self).get_channel(index)
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        (**self).iter_frames()
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        (**self).iter_channels()
    }

    fn channels(&self) -> usize {
        (**self).channels()
    }

    fn samples(&self) -> usize {
        (**self).samples()
    }

    fn sample_rate(&self) -> SampleRate {
        (**self).sample_rate()
    }
}
