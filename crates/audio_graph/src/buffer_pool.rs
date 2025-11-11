use std::{
    collections::{HashMap, VecDeque},
    num::NonZero,
};

use audio_buffer::{buffers::interleaved::InterleavedBuffer, core::Buffer};
use time::{FrameTime, SampleRate};

pub struct BufferArena<T> {
    // (channels, buffer_size)
    free: HashMap<(usize, FrameTime), VecDeque<InterleavedBuffer<T>>>,
    sample_rate: SampleRate,
}

impl<T> BufferArena<T>
where
    T: audio_buffer::dasp::Sample,
{
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            free: HashMap::new(),
            sample_rate,
        }
    }

    pub fn allocate_buffer(&mut self, channels: usize, buffer_size: FrameTime) {
        match self.free.get_mut(&(channels, buffer_size)) {
            Some(queue) => queue.push_front(InterleavedBuffer::with_shape(
                NonZero::new(channels).unwrap(),
                self.sample_rate,
                buffer_size,
            )),
            None => {
                let mut queue = VecDeque::new();
                queue.push_front(InterleavedBuffer::with_shape(
                    NonZero::new(channels).unwrap(),
                    self.sample_rate,
                    buffer_size,
                ));
                self.free.insert((channels, buffer_size), queue);
            }
        }
    }

    pub fn ensure_capacity(&mut self, channels: usize, buffer_size: FrameTime, amount: usize) {
        let required = match self.free.get(&(channels, buffer_size)) {
            Some(queue) => amount - queue.len(),
            None => amount,
        };

        for _ in 0..required {
            self.allocate_buffer(channels, buffer_size);
        }
    }

    pub fn take(
        &mut self,
        channels: usize,
        buffer_size: FrameTime,
    ) -> Option<InterleavedBuffer<T>> {
        match self.free.get_mut(&(channels, buffer_size)) {
            Some(queue) => match queue.pop_front() {
                Some(buffer) => Some(buffer),
                None => None,
            },
            None => None,
        }
    }

    pub fn release(&mut self, buffer: InterleavedBuffer<T>) {
        let num_channels = buffer.channels();
        let size = buffer.frames();
        match self.free.get_mut(&(num_channels, size.into())) {
            Some(queue) => queue.push_back(buffer),
            None => {
                let mut queue = VecDeque::new();
                queue.push_front(buffer);
                self.free.insert((num_channels, size.into()), queue);
            }
        }
    }
}
