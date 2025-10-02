use std::default;

use lending_iterator::{FromFn, prelude::HKT};

use crate::{
    buffers::{
        fixed_frames::iter::{ChannelIter, FrameIter},
        view::{Index, IndexMut, InjectiveFn, MutableView, View},
    },
    core::{Buffer, BufferMut},
};

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
    sample_rate: usize,
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
        = ChannelIter<'this, T, F>
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
        ChannelIter::new(self, 0)
    }

    fn channels(&self) -> usize {
        self.data.len()
    }

    fn samples(&self) -> usize {
        self.channels() * F
    }

    fn sample_rate(&self) -> usize {
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

    type IterFramesMut<'this>
    where
        Self: 'this;

    type IterChannelsMut<'this>
    where
        Self: 'this;

    fn iter_frames_mut(&mut self) -> Self::IterFramesMut<'_> {
        let mut position = 0;
        Box::new(lending_iterator::FromFn::<HKT!(Self::FrameMut<'_>), _, _> {
            state: &mut self.data,
            next: move |data| {
                Some(MutableView::new(
                    data,
                    InjectiveFn(Box::new(|channel: usize| (channel, position))),
                ))
            },
            _phantom: <_>::default(),
        })
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        todo!()
    }

    fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(Self::FrameMut<'this>) -> R,
    {
        if index < FRAMES {
            let frame = MutableView::new(
                &mut self.data,
                InjectiveFn(Box::new(move |channel: usize| (channel, index))),
            );
            Some(f(frame))
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
