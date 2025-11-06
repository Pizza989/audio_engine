use crate::{
    buffers::{
        fixed_frames::iter::FrameIter,
        view::{Index, IndexMut, InjectiveFn, MutableView, View},
    },
    core::{Buffer, BufferMut},
};
use time::SampleRate;

pub mod iter;

impl<T, const F: usize> Index<(usize, usize)> for Vec<[T; F]> {
    type Output = T;

    fn get_indexed(&self, index: (usize, usize)) -> Option<&Self::Output> {
        if index.0 < self.len() {
            Some(&self[index.0][index.1])
        } else {
            None
        }
    }
}

impl<T, const F: usize> IndexMut<(usize, usize)> for Vec<[T; F]> {
    fn get_indexed_mut(&mut self, index: (usize, usize)) -> Option<&mut Self::Output> {
        if index.0 < self.len() {
            Some(&mut self[index.0][index.1])
        } else {
            None
        }
    }
}

pub struct FixedFrameBuffer<T, const F: usize> {
    data: Vec<[T; F]>,
    sample_rate: SampleRate,
}

impl<T: dasp::Sample, const F: usize> FixedFrameBuffer<T, F> {
    pub fn with_capacity(channels: usize, sample_rate: SampleRate) -> Self {
        Self {
            data: vec![[T::EQUILIBRIUM; F]; channels],
            sample_rate,
        }
    }
}

impl<T: dasp::Sample, const F: usize> Buffer for FixedFrameBuffer<T, F> {
    type Sample = T;

    type Frame<'this>
        = View<'this, Vec<[T; F]>, usize, (usize, usize)>
    where
        Self: 'this;

    type Channel<'this>
        = &'this [T; F]
    where
        Self: 'this;

    type IterFrames<'this>
        = FrameIter<'this, T, F>
    where
        Self: 'this;

    type IterChannels<'this>
        = std::slice::Iter<'this, [T; F]>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        if index < F {
            Some(View::new(
                &self.data,
                Box::new(move |channel: usize| (channel, index)),
            ))
        } else {
            None
        }
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        if index < self.channels() {
            self.data.get(index)
        } else {
            None
        }
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        FrameIter::new(self, 0)
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        self.data.iter()
    }

    fn channels(&self) -> usize {
        self.data.len()
    }

    fn samples(&self) -> usize {
        self.channels() * F
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

impl<T: dasp::Sample, const FRAMES: usize> BufferMut for FixedFrameBuffer<T, FRAMES> {
    type FrameMut<'this>
        = MutableView<'this, Vec<[T; FRAMES]>, usize, (usize, usize)>
    where
        Self: 'this;

    type ChannelMut<'this>
        = &'this mut [T; FRAMES]
    where
        Self: 'this;

    fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'this>) -> R,
    {
        if index < FRAMES {
            // SAFETY: This is safe because the mapper that is passed is an injective function
            let view = unsafe {
                MutableView::from_raw(
                    &mut self.data as *mut _,
                    InjectiveFn(Box::new(move |channel: usize| (channel, index))),
                )
            };
            Some(f(view))
        } else {
            None
        }
    }

    fn with_channel_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::ChannelMut<'this>) -> R,
    {
        match self.data.get_mut(index) {
            Some(channel) => Some(f(channel)),
            None => None,
        }
    }
}
