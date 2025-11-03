use crate::buffers::{interleaved::InterleavedBuffer, view::View};
use crate::core::Buffer;

pub struct ChannelIter<'a, T: dasp::Sample> {
    buffer: &'a InterleavedBuffer<T>,
    position: usize,
}

impl<'a, T: dasp::Sample> ChannelIter<'a, T> {
    pub fn new(buffer: &'a InterleavedBuffer<T>, position: usize) -> Self {
        Self { buffer, position }
    }
}

impl<'a, T: dasp::Sample> Iterator for ChannelIter<'a, T> {
    type Item = View<'a, Vec<T>, usize, usize>;

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
