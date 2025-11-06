use dasp::Sample;

use crate::core::{
    Buffer, BufferMut,
    axis::{BufferAxis, BufferAxisMut},
    io::error::IoError,
};

pub mod error;
pub mod writer;

pub fn mix_buffers<T: Sample, I: Buffer<Sample = T>, O: BufferMut<Sample = T>>(
    input: &I,
    output: &mut O,
) -> Result<usize, IoError> {
    if input.channels() != output.channels() {
        return Err(IoError::ChannelMismatch(
            output.channels(),
            input.channels(),
        ));
    }

    let mut written = 0;

    output.map_frames_mut(
        |mut out_frame, frame_index| -> Option<()> {
            match input.get_frame(frame_index) {
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
        None,
    );

    Ok(written)
}
