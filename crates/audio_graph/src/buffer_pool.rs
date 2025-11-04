use std::collections::{HashMap, VecDeque};

use audio_buffer::buffers::fixed_frames::FixedFrameBuffer;

pub struct BufferPool<T, const BLOCK_SIZE: usize> {
    free: HashMap<usize, VecDeque<FixedFrameBuffer<T, BLOCK_SIZE>>>,
    sample_rate: usize,
}

impl<T, const BLOCK_SIZE: usize> BufferPool<T, BLOCK_SIZE>
where
    T: audio_buffer::dasp::Sample,
{
    pub fn new(sample_rate: usize) -> Self {
        Self {
            free: HashMap::new(),
            sample_rate,
        }
    }

    pub fn allocate_buffer(&mut self, channels: usize) {
        match self.free.get_mut(&channels) {
            Some(queue) => queue.push_front(FixedFrameBuffer::<T, BLOCK_SIZE>::with_capacity(
                channels,
                self.sample_rate,
            )),
            None => {
                let mut queue = VecDeque::new();
                queue.push_front(FixedFrameBuffer::<T, BLOCK_SIZE>::with_capacity(
                    channels,
                    self.sample_rate,
                ));
                self.free.insert(channels, queue);
            }
        }
    }

    pub fn aquire(&mut self, channels: usize) -> FixedFrameBuffer<T, BLOCK_SIZE> {
        match self.free.get_mut(&channels) {
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

    pub fn ensure_capacity(&mut self, channels: usize, amount: usize) {
        let required = match self.free.get(&channels) {
            Some(queue) => amount - queue.len(),
            None => amount,
        };

        for _ in 0..required {
            self.allocate_buffer(channels);
        }
    }
}
