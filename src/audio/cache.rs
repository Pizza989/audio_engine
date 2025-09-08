use slotmap::{SlotMap, new_key_type};
use symphonia::core::sample::Sample;

use crate::audio::shared_buffer::SharedBuffer;

new_key_type! { pub struct BufferKey; }

#[derive(Clone)]
pub struct AudioBufferCache<S: Sample> {
    pub buffers: SlotMap<BufferKey, SharedBuffer<S>>,
}

impl<S: Sample> AudioBufferCache<S> {
    pub fn new() -> Self {
        Self {
            buffers: SlotMap::with_key(),
        }
    }

    pub fn from_slotmap(buffers: SlotMap<BufferKey, SharedBuffer<S>>) -> Self {
        Self { buffers }
    }

    pub fn insert(mut self, buffer: SharedBuffer<S>) -> (AudioBufferCache<S>, BufferKey) {
        let key = self.buffers.insert(buffer);
        (self, key)
    }

    pub fn get(&self, key: BufferKey) -> Option<&SharedBuffer<S>> {
        self.buffers.get(key)
    }
}
