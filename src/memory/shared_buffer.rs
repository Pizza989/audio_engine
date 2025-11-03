use std::sync::Arc;

use audio_buffer::core::Buffer;

pub struct SharedBuffer<B: Buffer>(Arc<B>);

impl<B: Buffer> SharedBuffer<B> {
    pub fn new(buffer: B) -> SharedBuffer<B> {
        Self(Arc::new(buffer))
    }
}

impl<B: Buffer> Clone for SharedBuffer<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<B: Buffer> Buffer for SharedBuffer<B> {
    type Sample = B::Sample;

    type Frame<'this>
        = B::Frame<'this>
    where
        Self: 'this;

    type Channel<'this>
        = B::Channel<'this>
    where
        Self: 'this;

    type IterFrames<'this>
        = B::IterFrames<'this>
    where
        Self: 'this;

    type IterChannels<'this>
        = B::IterChannels<'this>
    where
        Self: 'this;

    fn get_frame(&self, index: usize) -> Option<Self::Frame<'_>> {
        self.0.get_frame(index)
    }

    fn get_channel(&self, index: usize) -> Option<Self::Channel<'_>> {
        self.0.get_channel(index)
    }

    fn iter_frames(&self) -> Self::IterFrames<'_> {
        self.0.iter_frames()
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        self.0.iter_channels()
    }

    fn channels(&self) -> usize {
        self.0.channels()
    }

    fn samples(&self) -> usize {
        self.0.samples()
    }

    fn sample_rate(&self) -> usize {
        self.0.sample_rate()
    }
}
