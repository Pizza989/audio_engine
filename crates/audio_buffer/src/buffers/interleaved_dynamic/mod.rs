use self::iter::{ChannelIter, ChannelIterMut, FrameIter, FrameIterMut};
use crate::core::{
    Buffer, BufferMut, ResizableBuffer,
    stride::{StridedSlice, StridedSliceMut},
};

pub mod iter;

pub struct InterleavedDynamicBuffer<T> {
    data: Vec<T>,
    channels: usize,
    sample_rate: usize,
}

impl<T> InterleavedDynamicBuffer<T> {
    pub fn new(channels: usize, sample_rate: usize) -> Self {
        Self {
            data: Vec::<T>::new(),
            channels,
            sample_rate,
        }
    }

    pub fn with_topology(channels: usize, sample_rate: usize, capacity: usize) -> Self {
        Self {
            data: Vec::<T>::with_capacity(capacity),
            channels,
            sample_rate,
        }
    }
}

impl<T: dasp::Sample> Buffer for InterleavedDynamicBuffer<T> {
    type Sample = T;

    type Frame<'this>
        = &'this [Self::Sample]
    where
        Self: 'this;

    type Channel<'this>
        = StridedSlice<'this, Self::Sample>
    where
        Self: 'this;

    type IterFrames<'this>
        = FrameIter<'this, T>
    where
        Self: 'this;

    type IterChannels<'this>
        = ChannelIter<'this, T>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        self.data.get(index..index + self.channels())
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        if index < self.channels() {
            Some(unsafe {
                StridedSlice::new(
                    &self.data,
                    index,
                    self.samples() / self.channels(),
                    self.channels(),
                )
            })
        } else {
            None
        }
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        FrameIter::new(self.data.chunks_exact(self.channels()))
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(
            &self.data,
            0,
            self.samples() / self.channels(),
            self.channels(),
        )
    }

    fn samples(&self) -> usize {
        self.data.len()
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

impl<T: dasp::Sample + 'static> BufferMut for InterleavedDynamicBuffer<T> {
    type FrameMut<'this>
        = &'this mut [Self::Sample]
    where
        Self: 'this;

    type ChannelMut<'this>
        = StridedSliceMut<'this, Self::Sample>
    where
        Self: 'this;

    type IterFramesMut<'this>
        = FrameIterMut<'this, T>
    where
        Self: 'this;

    type IterChannelsMut<'this>
        = ChannelIterMut<'this, T>
    where
        Self: 'this;

    fn iter_frames_mut(&mut self) -> Self::IterFramesMut<'_> {
        let channels = self.channels();
        FrameIterMut::new(self.data.chunks_exact_mut(channels))
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        ChannelIterMut::new(
            self.data.as_mut_ptr(),
            0,
            self.samples() / self.channels(),
            self.channels(),
        )
    }

    fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'this>) -> R,
    {
        let channels = self.channels();
        match self.data.get_mut(index..index + channels) {
            Some(frame) => Some(f(frame)),
            None => None,
        }
    }

    fn with_channel_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'this>) -> R,
    {
        let samples = self.samples();
        let channels = self.channels();
        if index < self.channels() {
            Some(f(unsafe {
                StridedSliceMut::new(&mut self.data, index, samples / channels, channels)
            }))
        } else {
            None
        }
    }
}

impl<T: dasp::Sample> ResizableBuffer for InterleavedDynamicBuffer<T> {
    fn resize(&mut self, frames: usize) {
        self.data.resize(frames * self.channels, T::EQUILIBRIUM);
    }
}
