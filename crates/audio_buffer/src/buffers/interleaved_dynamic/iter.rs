use std::marker::PhantomData;

use crate::core::stride::{StridedSlice, StridedSliceMut};

pub struct FrameIter<'a, T> {
    data: std::slice::ChunksExact<'a, T>,
}

impl<'a, T> FrameIter<'a, T> {
    pub fn new(data: std::slice::ChunksExact<'a, T>) -> Self {
        Self { data }
    }
}

impl<'a, T> Iterator for FrameIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for FrameIter<'a, T> {
    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct FrameIterMut<'a, T> {
    data: std::slice::ChunksExactMut<'a, T>,
}

impl<'a, T> FrameIterMut<'a, T> {
    pub fn new(data: std::slice::ChunksExactMut<'a, T>) -> Self {
        Self { data }
    }
}

impl<'a, T> Iterator for FrameIterMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for FrameIterMut<'a, T> {
    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct ChannelIter<'a, T> {
    data: &'a [T],
    channels: usize,
    channel_index: usize,
    samples_per_channel: usize,
}

impl<'a, T> ChannelIter<'a, T> {
    pub fn new(
        data: &'a [T],
        channel_index: usize,
        channels: usize,
        samples_per_channel: usize,
    ) -> Self {
        Self {
            data,
            channel_index,
            samples_per_channel,
            channels,
        }
    }
}

impl<'a, T> Iterator for ChannelIter<'a, T> {
    type Item = StridedSlice<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel_index < self.channels {
            let strided_slice = unsafe {
                StridedSlice::new(
                    self.data,
                    self.channel_index,
                    self.samples_per_channel,
                    self.channels,
                )
            };
            self.channel_index += 1;
            Some(strided_slice)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.channels - self.channel_index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for ChannelIter<'a, T> {
    fn len(&self) -> usize {
        self.channels - self.channel_index
    }
}

pub struct ChannelIterMut<'a, T> {
    data: *mut T,
    channel_index: usize,
    channels: usize,
    samples_per_channel: usize,
    _life: PhantomData<&'a mut [T]>,
}

impl<'a, T> ChannelIterMut<'a, T> {
    pub fn new(
        data: *mut T,
        channel_index: usize,
        channels: usize,
        samples_per_channel: usize,
    ) -> Self {
        Self {
            data,
            channel_index,
            samples_per_channel,
            _life: PhantomData,
            channels,
        }
    }
}

impl<'a, T> Iterator for ChannelIterMut<'a, T> {
    type Item = StridedSliceMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel_index < self.channels {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.data, self.samples_per_channel * self.channels)
            };
            let strided_slice_mut = unsafe {
                StridedSliceMut::new(
                    slice,
                    self.channel_index,
                    self.samples_per_channel,
                    self.channels,
                )
            };
            self.channel_index += 1;
            Some(strided_slice_mut)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.channels - self.channel_index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for ChannelIterMut<'a, T> {
    fn len(&self) -> usize {
        self.channels - self.channel_index
    }
}
