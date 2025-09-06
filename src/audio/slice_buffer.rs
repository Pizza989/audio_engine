use audio::{Buf, ExactSizeBuf, channel::InterleavedChannel};

pub struct SliceBuffer<'a, S: Copy> {
    data: &'a [S],
    channels: usize,
}

impl<'a, S: Copy> SliceBuffer<'a, S> {
    pub fn from_slice(data: &'a [S], channels: usize) -> Self {
        Self { data, channels }
    }
}

impl<'a, S: Copy> Buf for SliceBuffer<'a, S> {
    type Sample = S;

    type Channel<'this>
        = InterleavedChannel<'this, Self::Sample>
    where
        Self: 'this;

    type IterChannels<'this>
        = SliceBufferIterChannels<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        Some(self.data.len())
    }

    #[inline]
    fn channels(&self) -> usize {
        self.channels
    }

    #[inline]
    fn get_channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        InterleavedChannel::from_slice(&self.data, channel, self.channels)
    }

    #[inline]
    fn iter_channels(&self) -> Self::IterChannels<'_> {
        SliceBufferIterChannels::new(self)
    }
}

impl<'a, S: Copy> ExactSizeBuf for SliceBuffer<'a, S> {
    fn frames(&self) -> usize {
        self.data.len()
    }
}

pub struct SliceBufferIterChannels<'a, S: Copy> {
    buffer: &'a SliceBuffer<'a, S>,
    channel: usize,
}

impl<'a, S: Copy> SliceBufferIterChannels<'a, S> {
    pub fn new(buffer: &'a SliceBuffer<'a, S>) -> Self {
        Self { buffer, channel: 0 }
    }
}

impl<'a, S: Copy> Iterator for SliceBufferIterChannels<'a, S> {
    type Item = InterleavedChannel<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == self.buffer.channels {
            return None;
        }

        let channel = self.channel;
        self.channel += 1;

        Some(InterleavedChannel::from_slice(
            &self.buffer.data,
            channel,
            self.buffer.channels,
        )?)
    }
}
