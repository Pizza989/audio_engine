use crate::{
    core::{
        Buffer, BufferMut,
        stride::{StridedSlice, StridedSliceMut},
    },
    interleaved_dynamic::iter::{ChannelIter, ChannelIterMut, FrameIter, FrameIterMut},
};

pub struct WrapInterleaved<'a, T> {
    data: &'a [T],
    channels: usize,
}

impl<'a, T> WrapInterleaved<'a, T> {
    pub fn new(data: &'a [T], channels: usize) -> Self {
        Self { data, channels }
    }
}

impl<'a, T: dasp::Sample> Buffer for WrapInterleaved<'a, T> {
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
        = FrameIter<'this, Self::Sample>
    where
        Self: 'this;

    type IterChannels<'this>
        = ChannelIter<'this, Self::Sample>
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
        FrameIter::new(self.data.chunks_exact(self.channels()), self.channels())
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(
            &self.data,
            0,
            self.samples() / self.channels(),
            self.channels(),
        )
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> usize {
        self.data.len()
    }
}

pub struct WrapInterleavedMut<'a, T> {
    data: &'a mut [T],
    channels: usize,
}

impl<'a, T> WrapInterleavedMut<'a, T> {
    pub fn new(data: &'a mut [T], channels: usize) -> Self {
        Self { data, channels }
    }
}

impl<'a, T: dasp::Sample> Buffer for WrapInterleavedMut<'a, T> {
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
        = FrameIter<'this, Self::Sample>
    where
        Self: 'this;

    type IterChannels<'this>
        = ChannelIter<'this, Self::Sample>
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
        FrameIter::new(self.data.chunks_exact(self.channels()), self.channels())
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(
            &self.data,
            0,
            self.samples() / self.channels(),
            self.channels(),
        )
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> usize {
        self.data.len()
    }
}

impl<'a, T: dasp::Sample + 'static> BufferMut for WrapInterleavedMut<'a, T> {
    type FrameMut<'this>
        = &'this mut [T]
    where
        Self: 'this;

    type ChannelMut<'this>
        = StridedSliceMut<'this, T>
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
        FrameIterMut::new(self.data.chunks_exact_mut(self.channels), self.channels)
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        ChannelIterMut::new(
            self.data.as_mut_ptr(),
            0,
            self.channels,
            self.samples() / self.channels,
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
