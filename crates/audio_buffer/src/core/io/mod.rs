use crate::core::{
    Buffer, BufferMut,
    axis::{BufferAxis, BufferAxisMut},
};

pub mod error;
pub mod writer;

// TODO: Fix the below regression
// Apparently you cannot iterate over the samples of a frame that was borrowed while iterating over a buffer
pub fn mix_buffers<T: dasp::Sample + 'static, I: Buffer<Sample = T>, O: BufferMut<Sample = T>>(
    input: &I,
    output: &mut O,
) -> usize {
    let mut written = 0;

    for (mut dst_frame, src_frame) in output.iter_frames_mut().zip(input.iter_frames()) {
        for (s, d) in src_frame.iter_samples().zip(dst_frame.iter_samples_mut()) {
            *d = d.add_amp(s.to_signed_sample());
        }
        written += 1;
    }

    written
}
