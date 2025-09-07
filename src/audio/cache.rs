use std::sync::Arc;

use audio::buf::Dynamic;
use slotmap::{SlotMap, new_key_type};
use symphonia::core::sample::Sample;

new_key_type! { pub struct BufferKey; }

#[derive(Clone)]
pub struct AudioBufferCache<S: Sample> {
    pub buffers: SlotMap<BufferKey, Arc<Dynamic<S>>>,
}

impl<S: Sample> AudioBufferCache<S> {
    pub fn new() -> Self {
        Self {
            buffers: SlotMap::with_key(),
        }
    }

    pub fn from_slotmap(buffers: SlotMap<BufferKey, Arc<Dynamic<S>>>) -> Self {
        Self { buffers }
    }

    pub fn insert(mut self, buffer: Arc<Dynamic<S>>) -> (AudioBufferCache<S>, BufferKey) {
        let key = self.buffers.insert(buffer);
        (self, key)
    }

    pub fn get(&self, key: BufferKey) -> Option<&Arc<Dynamic<S>>> {
        self.buffers.get(key)
    }
}
