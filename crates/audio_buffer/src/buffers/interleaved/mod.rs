use std::num::NonZeroUsize;

use time::{FrameTime, SampleRate};

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
    sample_rate: SampleRate,
}

impl<T: dasp::Sample> InterleavedBuffer<T> {
    pub fn new(channels: NonZeroUsize, sample_rate: SampleRate) -> Self {
        Self {
            data: Vec::<T>::new(),
            channels,
            sample_rate,
        }
    }

    pub fn with_shape(channels: NonZeroUsize, sample_rate: SampleRate, frames: FrameTime) -> Self {
        Self {
            data: vec![T::EQUILIBRIUM; (frames * channels.get() as u64).0 as usize],
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

    fn sample_rate(&self) -> SampleRate {
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

    fn set_to_equilibrium(&mut self) {
        self.data.fill(T::EQUILIBRIUM);
    }
}

impl<T: dasp::Sample> ResizableBuffer for InterleavedBuffer<T> {
    fn resize(&mut self, frames: usize) {
        self.data.resize(frames * self.channels(), T::EQUILIBRIUM);
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use dasp::Sample;
    use time::SampleRate;

    use crate::{
        buffers::interleaved::InterleavedBuffer,
        core::{Buffer, BufferMut, ResizableBuffer, axis::BufferAxisMut},
    };

    #[test]
    fn with_shape_is_full() {
        let buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(256),
        );
        assert_eq!(buffer.samples(), 512);
        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.frames(), 256);
    }

    #[test]
    fn new_buffer_is_empty() {
        let buffer = InterleavedBuffer::<f32>::new(NonZero::new(2).unwrap(), SampleRate::default());
        assert_eq!(buffer.samples(), 0);
        assert_eq!(buffer.frames(), 0);
        assert_eq!(buffer.channels(), 2);
    }

    #[test]
    fn get_frame_returns_correct_slice() {
        let buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(3),
        );

        let frame0 = buffer.get_frame(0).unwrap();
        assert_eq!(frame0.len(), 2);

        let frame_out_of_bounds = buffer.get_frame(10);
        assert!(frame_out_of_bounds.is_none());
    }

    #[test]
    fn get_channel_returns_correct_view() {
        let buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(2),
        );

        let channel0 = buffer.get_channel(0).unwrap();
        let channel1 = buffer.get_channel(1).unwrap();
        assert!(channel0.get(0).is_some());
        assert!(channel1.get(0).is_some());

        assert!(channel0.get(10).is_none());
        assert!(channel1.get(10).is_none());

        assert!(buffer.get_channel(2).is_none());
    }

    #[test]
    fn with_frame_mut_changes_values() {
        let mut buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(1),
        );

        buffer.with_frame_mut(0, |frame| {
            frame[0] = 1.0;
            frame[1] = 2.0;
        });

        let frame = buffer.get_frame(0).unwrap();
        assert_eq!(frame, &[1.0, 2.0]);
    }

    #[test]
    fn with_channel_mut_changes_values() {
        let mut buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(2),
        );

        buffer.with_channel_mut(0, |mut channel| {
            channel.map_samples_mut(
                |sample, _| {
                    *sample = 42.0;
                    Some(())
                },
                None,
            );
        });

        let channel0 = buffer.get_channel(0).unwrap();

        assert_eq!(
            &[*channel0.get(0).unwrap(), *channel0.get(1).unwrap()],
            &[42.0, 42.0]
        );
    }

    #[test]
    fn iter_frames_returns_correct_chunks() {
        let buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(3),
        );

        let frames: Vec<_> = buffer.iter_frames().collect();
        assert_eq!(frames.len(), 3);
        assert!(frames.iter().all(|f| f.len() == 2));
    }

    #[test]
    fn resize_changes_frames() {
        let mut buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(2),
        );

        buffer.resize(4);
        assert_eq!(buffer.frames(), 4);
        assert_eq!(buffer.samples(), 8);
    }

    #[test]
    fn set_to_equilibrium_fills_buffer() {
        let mut buffer = InterleavedBuffer::<f32>::with_shape(
            NonZero::new(2).unwrap(),
            SampleRate::default(),
            time::FrameTime(3),
        );

        buffer.set_to_equilibrium();
        assert!(buffer.data.iter().all(|&s| s == f32::EQUILIBRIUM));
    }
}
