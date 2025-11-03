use std::num::NonZeroUsize;

use self::iter::ChannelIter;
use crate::{
    buffers::view::{Index, IndexMut, InjectiveFn, MutableView, View},
    core::{Buffer, BufferMut, ResizableBuffer},
};

pub mod iter;

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    fn get_indexed(&self, index: usize) -> Option<&Self::Output> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn get_indexed_mut(&mut self, index: usize) -> Option<&mut Self::Output> {
        if index < self.len() {
            Some(&mut self[index])
        } else {
            None
        }
    }
}

pub struct InterleavedBuffer<T> {
    data: Vec<T>,
    channels: NonZeroUsize,
    sample_rate: usize,
}

impl<T> InterleavedBuffer<T> {
    pub fn new(channels: NonZeroUsize, sample_rate: usize) -> Self {
        Self {
            data: Vec::<T>::new(),
            channels,
            sample_rate,
        }
    }

    pub fn with_capacity(channels: NonZeroUsize, sample_rate: usize, capacity: usize) -> Self {
        Self {
            data: Vec::<T>::with_capacity(capacity),
            channels,
            sample_rate,
        }
    }
}

impl<T: dasp::Sample> Buffer for InterleavedBuffer<T> {
    type Sample = T;

    type Frame<'this>
        = &'this [Self::Sample]
    where
        Self: 'this;

    type Channel<'this>
        = View<'this, Vec<T>, usize, usize>
    where
        Self: 'this;

    type IterFrames<'this>
        = std::slice::ChunksExact<'this, T>
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
        let channels = self.channels();
        if index < self.channels() {
            Some(View::new(
                &self.data,
                Box::new(move |sample: usize| sample * channels + index),
            ))
        } else {
            None
        }
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        self.data.chunks_exact(self.channels())
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        ChannelIter::new(&self, 0)
    }

    fn samples(&self) -> usize {
        self.data.len()
    }

    fn channels(&self) -> usize {
        self.channels.into()
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

impl<T: dasp::Sample + 'static> BufferMut for InterleavedBuffer<T> {
    type FrameMut<'this>
        = &'this mut [Self::Sample]
    where
        Self: 'this;

    type ChannelMut<'this>
        = MutableView<'this, Vec<T>, usize, usize>
    where
        Self: 'this;

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
        let channels = self.channels();
        if index < self.channels() {
            // SAFETY: The function is injective as long as channels != 0
            Some(f(unsafe {
                MutableView::from_raw(
                    &mut self.data,
                    InjectiveFn(Box::new(move |sample| sample * channels + index)),
                )
            }))
        } else {
            None
        }
    }
}

impl<T: dasp::Sample> ResizableBuffer for InterleavedBuffer<T> {
    fn resize(&mut self, frames: usize) {
        self.data.resize(frames * self.channels(), T::EQUILIBRIUM);
    }
}
