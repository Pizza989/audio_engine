use std::marker::PhantomData;

use crate::core::stride::{StridedSlice, StridedSliceMut};

pub struct FrameIter<'a, T, const CHANNELS: usize> {
    data: std::slice::ChunksExact<'a, T>,
}

impl<'a, T, const CHANNELS: usize> FrameIter<'a, T, CHANNELS> {
    pub fn new(data: std::slice::ChunksExact<'a, T>) -> Self {
        Self { data }
    }
}

impl<'a, T, const CHANNELS: usize> Iterator for FrameIter<'a, T, CHANNELS> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, T, const CHANNELS: usize> ExactSizeIterator for FrameIter<'a, T, CHANNELS> {
    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct FrameIterMut<'a, T, const CHANNELS: usize> {
    data: std::slice::ChunksExactMut<'a, T>,
}

impl<'a, T, const CHANNELS: usize> FrameIterMut<'a, T, CHANNELS> {
    pub fn new(data: std::slice::ChunksExactMut<'a, T>) -> Self {
        Self { data }
    }
}

impl<'a, T, const CHANNELS: usize> Iterator for FrameIterMut<'a, T, CHANNELS> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, T, const CHANNELS: usize> ExactSizeIterator for FrameIterMut<'a, T, CHANNELS> {
    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct ChannelIter<'a, T, const CHANNELS: usize> {
    data: &'a [T],
    channel_index: usize,
    samples_per_channel: usize,
}

impl<'a, T, const CHANNELS: usize> ChannelIter<'a, T, CHANNELS> {
    pub fn new(data: &'a [T], channel_index: usize, samples_per_channel: usize) -> Self {
        Self {
            data,
            channel_index,
            samples_per_channel,
        }
    }
}

impl<'a, T, const CHANNELS: usize> Iterator for ChannelIter<'a, T, CHANNELS> {
    type Item = StridedSlice<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel_index < CHANNELS {
            let strided_slice = unsafe {
                StridedSlice::new(
                    self.data,
                    self.channel_index,
                    self.samples_per_channel,
                    CHANNELS,
                )
            };
            self.channel_index += 1;
            Some(strided_slice)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = CHANNELS - self.channel_index;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const CHANNELS: usize> ExactSizeIterator for ChannelIter<'a, T, CHANNELS> {
    fn len(&self) -> usize {
        CHANNELS - self.channel_index
    }
}

pub struct ChannelIterMut<'a, T, const CHANNELS: usize> {
    data: *mut T,
    channel_index: usize,
    samples_per_channel: usize,
    _life: PhantomData<&'a mut [T]>,
}

impl<'a, T, const CHANNELS: usize> ChannelIterMut<'a, T, CHANNELS> {
    pub fn new(data: *mut T, channel_index: usize, samples_per_channel: usize) -> Self {
        Self {
            data,
            channel_index,
            samples_per_channel,
            _life: PhantomData,
        }
    }
}

impl<'a, T, const CHANNELS: usize> Iterator for ChannelIterMut<'a, T, CHANNELS> {
    type Item = StridedSliceMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel_index < CHANNELS {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(self.data, self.samples_per_channel * CHANNELS)
            };
            let strided_slice_mut = unsafe {
                StridedSliceMut::new(
                    slice,
                    self.channel_index,
                    self.samples_per_channel,
                    CHANNELS,
                )
            };
            self.channel_index += 1;
            Some(strided_slice_mut)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = CHANNELS - self.channel_index;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const CHANNELS: usize> ExactSizeIterator for ChannelIterMut<'a, T, CHANNELS> {
    fn len(&self) -> usize {
        CHANNELS - self.channel_index
    }
}
