// TODO:
// - rework to fix some of the problems
// - add option to write mismatching buffers by selecting channels
use crate::core::{
    Buffer, BufferMut, ResizableBuffer,
    axis::{BufferAxis, BufferAxisMut},
    io::error::IoError,
};

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
    pub fn write_block_remaining<I: Buffer<Sample = T>>(
        &mut self,
        input: &I,
    ) -> Result<usize, IoError> {
        if self.buffer.channels() != input.channels() {
            return Err(IoError::ChannelMismatch(
                self.buffer.channels(),
                input.channels(),
            ));
        }

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
        Ok(written)
    }

    pub fn mix_block_remaining<I: Buffer<Sample = T>>(
        &mut self,
        input: &I,
    ) -> Result<usize, IoError> {
        if self.buffer.channels() != input.channels() {
            return Err(IoError::ChannelMismatch(
                self.buffer.channels(),
                input.channels(),
            ));
        }
        let mut written = 0;

        for (mut dst_frame, src_frame) in self
            .buffer
            .iter_frames_mut()
            .skip(self.position)
            .zip(input.iter_frames())
        {
            for (s, d) in src_frame.iter_samples().zip(dst_frame.iter_samples_mut()) {
                *d = d.add_amp(s.to_signed_sample());
            }
            written += 1;
        }

        self.position += written;
        Ok(written)
    }
}

impl<'a, T: dasp::Sample + 'static, B: BufferMut<Sample = T> + ResizableBuffer> Writer<'a, T, B> {
    // PROBLEM: this will ensure the capacity even if passed a buffer with a channel mismatch
    pub fn write_block_growing<I: Buffer<Sample = T>>(
        &mut self,
        input: &I,
    ) -> Result<usize, IoError> {
        self.buffer.ensure_capacity(self.position + input.frames());
        self.write_block_remaining(input)
    }

    pub fn mix_block_growing<I: Buffer<Sample = T>>(
        &mut self,
        input: &I,
    ) -> Result<usize, IoError> {
        self.buffer.ensure_capacity(self.position + input.frames());
        self.mix_block_remaining(input)
    }
}
