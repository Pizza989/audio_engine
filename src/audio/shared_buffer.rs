use std::sync::Arc;

use audio::{
    Buf, ExactSizeBuf,
    buf::{Dynamic, dynamic::IterChannels},
    channel::LinearChannel,
};

#[derive(Clone)]
pub struct SharedBuffer<T>(Arc<Dynamic<T>>);

impl<T> SharedBuffer<T> {
    pub fn new(buffer: Dynamic<T>) -> SharedBuffer<T> {
        Self(Arc::new(buffer))
    }
}

impl<T: Copy> Buf for SharedBuffer<T> {
    type Sample = T;

    type Channel<'this>
        = LinearChannel<'this, Self::Sample>
    where
        Self: 'this;

    type IterChannels<'this>
        = IterChannels<'this, Self::Sample>
    where
        Self: 'this;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.0.frames())
    }

    fn channels(&self) -> usize {
        self.0.channels()
    }

    fn get_channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        (*self.0).get_channel(channel)
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        (*self.0).iter_channels()
    }
}

impl<T: Copy> ExactSizeBuf for SharedBuffer<T> {
    fn frames(&self) -> usize {
        self.0.frames()
    }
}
