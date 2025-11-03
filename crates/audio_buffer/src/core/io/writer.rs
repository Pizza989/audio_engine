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

        self.buffer.map_frames_mut(
            |mut out_frame, frame_index| -> Option<()> {
                match input.get_frame(frame_index - self.position) {
                    Some(in_frame) => {
                        let result = Some(out_frame.map_samples_mut(|out_sample, sample_index| {
                            match in_frame.get_sample(sample_index) {
                                Some(in_sample) => {
                                    *out_sample = *in_sample;
                                    Some(())
                                }
                                None => {
                                    unreachable!("channel mismatch between input and output buffers must not occur")
                                }
                        }
                    }));

                        written += 1;
                        result
                    }
                    None => None,
                }
            },
            Some(self.position),
        );

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

        self.buffer.map_frames_mut(
            |mut out_frame, frame_index| -> Option<()> {
                match input.get_frame(frame_index - self.position) {
                    Some(in_frame) => {
                        let result = Some(out_frame.map_samples_mut(|out_sample, sample_index| {
                    match in_frame.get_sample(sample_index) {
                        Some(in_sample) => {
                            *out_sample =
                                out_sample.add_amp(dasp::Sample::to_signed_sample(*in_sample));
                            Some(())
                        }
                        None => {
                            panic!(
                                "channel mismatch between input and output buffers must not occur"
                            )
                        }
                    }
                }));

                        written += 1;
                        result
                    }
                    None => None,
                }
            },
            Some(self.position),
        );

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
