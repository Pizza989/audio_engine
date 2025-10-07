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

            Some(unsafe {
                MutableView::from_raw(
                    &mut self.buffer.data as *mut _,
                    InjectiveFn(Box::new(move |channel: usize| (channel, position))),
                )
            })
        } else {
            None
        }
    }
}
