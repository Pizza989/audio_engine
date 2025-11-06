use std::{
    collections::{HashMap, VecDeque},
    num::NonZero,
};

use audio_buffer::buffers::interleaved::InterleavedBuffer;
use time::{FrameTime, SampleRate};

pub struct BufferPool<T> {
    // (channels, buffer_size)
    free: HashMap<(usize, FrameTime), VecDeque<InterleavedBuffer<T>>>,
    sample_rate: SampleRate,
}

impl<T> BufferPool<T>
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
            Some(queue) => queue.push_front(InterleavedBuffer::with_capacity(
                NonZero::new(channels).unwrap(),
                self.sample_rate,
                buffer_size,
            )),
            None => {
                let mut queue = VecDeque::new();
                queue.push_front(InterleavedBuffer::with_capacity(
                    NonZero::new(channels).unwrap(),
                    self.sample_rate,
                    buffer_size,
                ));
                self.free.insert((channels, buffer_size), queue);
            }
        }
    }

    pub fn aquire(&mut self, channels: usize, buffer_size: FrameTime) -> InterleavedBuffer<T> {
        match self.free.get_mut(&(channels, buffer_size)) {
            Some(queue) => match queue.pop_front() {
                Some(buffer) => buffer,
                None => panic!("Buffer Pool couldn't provide a buffer"),
            },
            None => panic!(
                "Buffer Pool didn't have a queue of buffers with {} channels",
                channels
            ),
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
}
