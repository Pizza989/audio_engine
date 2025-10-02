use crate::{
    buffers::{
        fixed_frames::FixedFrameBuffer,
        view::{InjectiveFn, MutableView, View},
    },
    core::Buffer,
};

pub struct FrameIter<'a, T: dasp::Sample, const F: usize> {
    buffer: &'a FixedFrameBuffer<T, F>,
    position: usize,
}

impl<'a, T: dasp::Sample, const F: usize> FrameIter<'a, T, F> {
    pub fn new(buffer: &'a FixedFrameBuffer<T, F>, position: usize) -> Self {
        Self { buffer, position }
    }
}

impl<'a, T: dasp::Sample, const F: usize> Iterator for FrameIter<'a, T, F> {
    type Item = View<'a, Vec<[T; F]>, usize, (usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < F {
            let position = self.position;
            self.position += 1;
            self.buffer.get_frame(position)
        } else {
            None
        }
    }
}

pub struct ChannelIter<'a, T: dasp::Sample, const F: usize> {
    buffer: &'a FixedFrameBuffer<T, F>,
    position: usize,
}

impl<'a, T: dasp::Sample, const F: usize> ChannelIter<'a, T, F> {
    pub fn new(buffer: &'a FixedFrameBuffer<T, F>, position: usize) -> Self {
        Self { buffer, position }
    }
}

impl<'a, T: dasp::Sample, const F: usize> Iterator for ChannelIter<'a, T, F> {
    type Item = &'a [T; F];

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.buffer.channels() {
            let position = self.position;
            self.position += 1;
            self.buffer.get_channel(position)
        } else {
            None
        }
    }
}

pub struct FrameIterMut<'a, T: dasp::Sample, const F: usize> {
    buffer: &'a mut FixedFrameBuffer<T, F>,
    position: usize,
}

impl<'a, T: dasp::Sample, const F: usize> FrameIterMut<'a, T, F> {
    pub fn new(buffer: &'a mut FixedFrameBuffer<T, F>, position: usize) -> Self {
        Self { buffer, position }
    }
}

impl<'a, T: dasp::Sample, const F: usize> Iterator for FrameIterMut<'a, T, F> {
    type Item = MutableView<'a, Vec<[T; F]>, usize, (usize, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < F {
            let position = self.position;
            self.position += 1;

            Some(MutableView::new(
                &mut self.buffer.data,
                InjectiveFn(Box::new(move |channel: usize| (channel, position))),
            ))
        } else {
            None
        }
    }
}

pub struct ChannelIterMut<'a, T: dasp::Sample, const F: usize> {
    buffer: &'a mut FixedFrameBuffer<T, F>,
    position: usize,
}

impl<'a, T: dasp::Sample, const F: usize> Iterator for ChannelIterMut<'a, T, F> {
    type Item = &'a mut [T; F];

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.buffer.channels() {
            let position = self.position;
            self.position += 1;

            Some(
                self.buffer
                    .data
                    .get_mut(position)
                    .expect("index is in bounds"),
            )
        } else {
            None
        }
    }
}
