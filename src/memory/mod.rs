use audio_buffer::core::Buffer;
use slotmap::{SlotMap, new_key_type};

pub mod shared_buffer;

new_key_type! { pub struct BufferKey; }

pub struct BufferStorage<B>
where
    B: Buffer,
{
    map: SlotMap<BufferKey, B>,
}

impl<B> BufferStorage<B>
where
    B: Buffer,
{
    pub fn new() -> Self {
        Self {
            map: SlotMap::with_key(),
        }
    }

    pub fn insert(&mut self, buffer: B) -> BufferKey {
        self.map.insert(buffer)
    }

    pub fn remove(&mut self, key: BufferKey) -> Option<B> {
        self.map.remove(key)
    }

    pub fn get(&self, key: BufferKey) -> Option<&B> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: BufferKey) -> Option<&mut B> {
        self.map.get_mut(key)
    }
}
