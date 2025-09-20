use crate::{
    core::{
        Buffer, BufferMut,
        stride::{StridedSlice, StridedSliceMut},
    },
    interleaved_static::iter::{ChannelIter, ChannelIterMut, FrameIter, FrameIterMut},
};

pub mod iter;

pub struct InterleavedStaticBuffer<T, const CHANNELS: usize, const SAMPLES: usize> {
    data: [T; SAMPLES],
}

impl<T: dasp::Sample, const CHANNELS: usize, const SAMPLES: usize>
    InterleavedStaticBuffer<T, CHANNELS, SAMPLES>
{
    pub fn new() -> Self {
        Self {
            data: [T::EQUILIBRIUM; SAMPLES],
        }
    }
}

impl<T: dasp::Sample, const CHANNELS: usize, const SAMPLES: usize> Buffer
    for InterleavedStaticBuffer<T, CHANNELS, SAMPLES>
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
        = ChannelIter<'this, Self::Sample, CHANNELS>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        self.data.get(index..index + CHANNELS)
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        if index < CHANNELS {
            Some(unsafe { StridedSlice::new(&self.data, index, SAMPLES / CHANNELS, CHANNELS) })
        } else {
            None
        }
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        FrameIter::new(self.data.chunks_exact(CHANNELS))
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(&self.data, 0, SAMPLES / CHANNELS)
    }

    fn samples(&self) -> usize {
        SAMPLES
    }

    fn channels(&self) -> usize {
        CHANNELS
    }
}

impl<T: dasp::Sample + 'static, const CHANNELS: usize, const SAMPLES: usize> BufferMut
    for InterleavedStaticBuffer<T, CHANNELS, SAMPLES>
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
        = ChannelIterMut<'this, Self::Sample, CHANNELS>
    where
        Self: 'this;

    fn iter_frames_mut(&mut self) -> Self::IterFramesMut<'_> {
        FrameIterMut::new(self.data.chunks_exact_mut(CHANNELS))
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        ChannelIterMut::new(self.data.as_mut_ptr(), 0, SAMPLES / CHANNELS)
    }

    fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'this>) -> R,
    {
        match self.data.get_mut(index..index + CHANNELS) {
            Some(frame) => Some(f(frame)),
            None => None,
        }
    }

    fn with_channel_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'this>) -> R,
    {
        if index < CHANNELS {
            Some(f(unsafe {
                StridedSliceMut::new(&mut self.data, index, SAMPLES / CHANNELS, CHANNELS)
            }))
        } else {
            None
        }
    }
}
