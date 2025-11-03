use crate::{
    buffers::{fixed_frames::FixedFrameBuffer, view::View},
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
