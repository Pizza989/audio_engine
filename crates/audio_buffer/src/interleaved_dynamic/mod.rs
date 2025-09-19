use crate::{
    core::{
        Buffer, BufferMut,
        stride::{StridedSlice, StridedSliceMut},
    },
    interleaved_static::iter::{ChannelIter, ChannelIterMut, FrameIter, FrameIterMut},
};

pub mod iter;

pub struct InterleavedDynamicBuffer<T, const CHANNELS: usize> {
    data: Vec<T>,
}

impl<T: dasp::Sample, const CHANNELS: usize> InterleavedDynamicBuffer<T, CHANNELS> {
    pub fn new() -> Self {
        Self {
            data: Vec::<T>::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::<T>::with_capacity(capacity),
        }
    }
}

impl<T: dasp::Sample, const CHANNELS: usize> Buffer<CHANNELS>
    for InterleavedDynamicBuffer<T, CHANNELS>
{
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
        = FrameIter<'this, T, CHANNELS>
    where
        Self: 'this;

    type IterChannels<'this>
        = ChannelIter<'this, T, CHANNELS>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        self.data.get(index..index + CHANNELS)
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        if index < CHANNELS {
            Some(unsafe {
                StridedSlice::new(&self.data, index, self.samples() / CHANNELS, CHANNELS)
            })
        } else {
            None
        }
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        FrameIter::new(self.data.chunks_exact(CHANNELS))
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(&self.data, 0, self.samples() / CHANNELS)
    }

    fn samples(&self) -> usize {
        self.data.len()
    }
}

impl<T: dasp::Sample + 'static, const CHANNELS: usize> BufferMut<CHANNELS>
    for InterleavedDynamicBuffer<T, CHANNELS>
{
    type FrameMut<'this>
        = &'this mut [Self::Sample]
    where
        Self: 'this;

    type ChannelMut<'this>
        = StridedSliceMut<'this, Self::Sample>
    where
        Self: 'this;

    type IterFramesMut<'this>
        = FrameIterMut<'this, T, CHANNELS>
    where
        Self: 'this;

    type IterChannelsMut<'this>
        = ChannelIterMut<'this, T, CHANNELS>
    where
        Self: 'this;

    fn iter_frames_mut(&mut self) -> Self::IterFramesMut<'_> {
        FrameIterMut::new(self.data.chunks_exact_mut(CHANNELS))
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        ChannelIterMut::new(self.data.as_mut_ptr(), 0, self.samples() / CHANNELS)
    }

    fn with_frame_mut<F, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'_>) -> R,
    {
        match self.data.get_mut(index..index + CHANNELS) {
            Some(frame) => Some(f(frame)),
            None => None,
        }
    }

    fn with_channel_mut<F, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'_>) -> R,
    {
        let samples = self.samples();
        if index < CHANNELS {
            Some(f(unsafe {
                StridedSliceMut::new(&mut self.data, index, samples / CHANNELS, CHANNELS)
            }))
        } else {
            None
        }
    }
}
