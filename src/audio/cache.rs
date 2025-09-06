use audio::buf::Dynamic;
use slotmap::{SlotMap, new_key_type};
use symphonia::core::sample::Sample;

new_key_type! { pub struct BufferKey; }

pub struct AudioBufferCache<S: Sample> {
    buffers: SlotMap<BufferKey, Dynamic<S>>,
}

impl<S: Sample> AudioBufferCache<S> {
    pub fn new() -> Self {
        Self {
            buffers: SlotMap::with_key(),
        }
    }

    pub fn insert(&mut self, buffer: Dynamic<S>) -> BufferKey {
        self.buffers.insert(buffer)
    }

    pub fn get(&self, key: BufferKey) -> Option<&Dynamic<S>> {
        self.buffers.get(key)
    }

    pub fn get_mut(&mut self, key: BufferKey) -> Option<&mut Dynamic<S>> {
        self.buffers.get_mut(key)
    }
}
