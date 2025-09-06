use std::{path::Path, sync::Arc};

use slotmap::{SlotMap, new_key_type};

use audio::buf::{Dynamic, Interleaved};
use symphonium::SymphoniumLoader;
use time::SampleRate;

pub mod decode;
pub mod error;

#[derive(Clone)]
pub struct AudioBuffer {
    buffer: Arc<Dynamic<f32>>,
    sample_rate: SampleRate,
}

impl AudioBuffer {
    pub fn new(buffer: Dynamic<f32>, sample_rate: SampleRate) -> Self {
        Self {
            buffer: Arc::new(buffer),
            sample_rate,
        }
    }

    pub fn from_path(path: impl AsRef<Path>, loader: &mut SymphoniumLoader) -> Self {
        loader.load(path, None, symphonium::ResampleQuality::High, None);
    }
}

new_key_type! { pub struct BufferKey; }

#[derive(Default)]
pub struct AudioCache {
    buffers: SlotMap<BufferKey, AudioBuffer>,
}

impl AudioCache {
    pub fn insert(&mut self, buffer: AudioBuffer) -> BufferKey {
        self.buffers.insert(buffer)
    }

    pub fn get(&self, key: BufferKey) -> Option<&AudioBuffer> {
        self.buffers.get(key)
    }

    pub fn get_mut(&mut self, key: BufferKey) -> Option<&mut AudioBuffer> {
        self.buffers.get_mut(key)
    }
}
