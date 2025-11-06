use time::SampleRate;

use crate::{
    buffers::view::{Index, View},
    core::Buffer,
};

pub struct ChannelIter<'a, T: dasp::Sample> {
    buffer: &'a WrapInterleaved<'a, T>,
    position: usize,
}

impl<'a, T: dasp::Sample> ChannelIter<'a, T> {
    pub fn new(buffer: &'a WrapInterleaved<T>, position: usize) -> Self {
        Self { buffer, position }
    }
}

impl<'a, T: dasp::Sample> Iterator for ChannelIter<'a, T> {
    type Item = View<'a, &'a [T], usize, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.position;
        let channels = self.buffer.channels();
        if self.position < channels {
            let channel = View::new(
                &self.buffer.data,
                Box::new(move |sample| sample * channels + position),
            );
            self.position += 1;
            Some(channel)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.channels() - self.position;
        (remaining, Some(remaining))
    }
}

impl<'a, T: dasp::Sample> ExactSizeIterator for ChannelIter<'a, T> {
    fn len(&self) -> usize {
        self.buffer.channels() - self.position
    }
}

impl<T> Index<usize> for &[T] {
    type Output = T;

    fn get_indexed(&self, index: usize) -> Option<&Self::Output> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }
}

// impl<T> IndexMut<usize> for [T] {
//     fn get_indexed_mut(&mut self, index: usize) -> Option<&mut Self::Output> {
//         if index < self.len() {
//             Some(&mut self[index])
//         } else {
//             None
//         }
//     }
// }

pub struct WrapInterleaved<'a, T> {
    data: &'a [T],
    channels: usize,
    sample_rate: SampleRate,
}

impl<'a, T> WrapInterleaved<'a, T> {
    pub fn new(data: &'a [T], channels: usize, sample_rate: SampleRate) -> Self {
        Self {
            data,
            channels,
            sample_rate,
        }
    }
}

impl<'a, T: dasp::Sample> Buffer for WrapInterleaved<'a, T> {
    type Sample = T;

    type Frame<'this>
        = &'this [Self::Sample]
    where
        Self: 'this;

    type Channel<'this>
        = View<'this, &'this [T], usize, usize>
    where
        Self: 'this;

    type IterFrames<'this>
        = std::slice::ChunksExact<'this, T>
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
        let channels = self.channels();
        if index < channels {
            Some(View::with_stride(&self.data, channels, index))
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

    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> usize {
        self.data.len()
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

// pub struct WrapInterleavedMut<'a, T> {
//     data: &'a mut [T],
//     channels: usize,
//     sample_rate: usize,
// }

// impl<'a, T> WrapInterleavedMut<'a, T> {
//     pub fn new(data: &'a mut [T], channels: usize, sample_rate: usize) -> Self {
//         Self {
//             data,
//             channels,
//             sample_rate,
//         }
//     }
// }

// impl<'a, T: dasp::Sample> Buffer for WrapInterleavedMut<'a, T> {
//     type Sample = T;

//     type Frame<'this>
//         = &'this [Self::Sample]
//     where
//         Self: 'this;

//     type Channel<'this>
//         = StridedSlice<'this, Self::Sample>
//     where
//         Self: 'this;

//     type IterFrames<'this>
//         = FrameIter<'this, Self::Sample>
//     where
//         Self: 'this;

//     type IterChannels<'this>
//         = ChannelIter<'this, Self::Sample>
//     where
//         Self: 'this;

//     fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
//         self.data.get(index..index + self.channels())
//     }

//     fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
//         if index < self.channels() {
//             Some(unsafe {
//                 StridedSlice::new(
//                     &self.data,
//                     index,
//                     self.samples() / self.channels(),
//                     self.channels(),
//                 )
//             })
//         } else {
//             None
//         }
//     }

//     fn iter_frames(&self) -> Self::IterFrames<'_> {
//         FrameIter::new(self.data.chunks_exact(self.channels()))
//     }

//     fn iter_channels(&self) -> Self::IterChannels<'_> {
//         ChannelIter::new(
//             &self.data,
//             0,
//             self.samples() / self.channels(),
//             self.channels(),
//         )
//     }

//     fn channels(&self) -> usize {
//         self.channels
//     }

//     fn samples(&self) -> usize {
//         self.data.len()
//     }

//     fn sample_rate(&self) -> usize {
//         self.sample_rate
//     }
// }

// impl<'a, T: dasp::Sample + 'static> BufferMut for WrapInterleavedMut<'a, T> {
//     type FrameMut<'this>
//         = &'this mut [T]
//     where
//         Self: 'this;

//     type ChannelMut<'this>
//         = StridedSliceMut<'this, T>
//     where
//         Self: 'this;

//     type IterFramesMut<'this>
//         = FrameIterMut<'this, T>
//     where
//         Self: 'this;

//     type IterChannelsMut<'this>
//         = ChannelIterMut<'this, T>
//     where
//         Self: 'this;

//     fn with_frame_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
//     where
//         F: FnOnce(Self::FrameMut<'this>) -> R,
//     {
//         let channels = self.channels();
//         match self.data.get_mut(index..index + channels) {
//             Some(frame) => Some(f(frame)),
//             None => None,
//         }
//     }

//     fn with_channel_mut<'this, F, R>(&'this mut self, index: usize, f: F) -> Option<R>
//     where
//         F: FnOnce(Self::ChannelMut<'this>) -> R,
//     {
//         let samples = self.samples();
//         let channels = self.channels();
//         if index < self.channels() {
//             Some(f(unsafe {
//                 StridedSliceMut::new(&mut self.data, index, samples / channels, channels)
//             }))
//         } else {
//             None
//         }
//     }
// }
