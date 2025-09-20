use audio_buffer::core::Buffer;
use slotmap::{SlotMap, new_key_type};

use crate::audio::shared_buffer::SharedBuffer;

new_key_type! { pub struct BufferKey; }

#[derive(Clone)]
pub struct AudioBufferCache<B: Buffer> {
    pub buffers: SlotMap<BufferKey, SharedBuffer<B>>,
}

impl<B: Buffer> AudioBufferCache<B> {
    pub fn new() -> Self {
        Self {
            buffers: SlotMap::with_key(),
        }
    }

    pub fn from_slotmap(buffers: SlotMap<BufferKey, SharedBuffer<B>>) -> Self {
        Self { buffers }
    }

    pub fn insert(mut self, buffer: SharedBuffer<B>) -> (AudioBufferCache<B>, BufferKey) {
        let key = self.buffers.insert(buffer);
        (self, key)
    }

    pub fn get(&self, key: BufferKey) -> Option<&SharedBuffer<B>> {
        self.buffers.get(key)
    }
}
