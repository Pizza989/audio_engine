use crate::core::{Buffer, BufferAxis, BufferAxisMut, BufferMut, ResizableBuffer};

pub struct Writer<'a, T: dasp::Sample + 'static, B: BufferMut<Sample = T>> {
    buffer: &'a mut B,
    position: usize,
}

impl<'a, T: dasp::Sample + 'static, B: BufferMut<Sample = T>> Writer<'a, T, B> {
    pub fn new(buffer: &'a mut B) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// This will append as much of the input buffer into this buffer as is possible
    /// without resizing it. If it is not sure that the Buffer has the required size
    /// and it should be resized, use `write_block_growing`. This only works for
    /// buffers implementing `DynamicBuffer`.
    pub fn write_block<I: Buffer<Sample = T>>(&mut self, input: &I) -> usize {
        let mut written = 0;
        for (mut dst_frame, src_frame) in self
            .buffer
            .iter_frames_mut()
            .skip(self.position)
            .zip(input.iter_frames())
        {
            for (s, d) in src_frame.iter_samples().zip(dst_frame.iter_samples_mut()) {
                *d = *s;
            }
            written += 1;
        }

        self.position += written;
        written
    }
}

impl<'a, T: dasp::Sample + 'static, B: BufferMut<Sample = T> + ResizableBuffer> Writer<'a, T, B> {
    pub fn write_block_growing<I: Buffer<Sample = T>>(&mut self, input: &I) -> usize {
        self.buffer.ensure_capacity(self.position + input.frames());
        self.write_block(input)
    }
}
