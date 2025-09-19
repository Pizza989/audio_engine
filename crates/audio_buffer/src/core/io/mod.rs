use crate::core::{Buffer, BufferAxis, BufferAxisMut, BufferMut};

// pub mod reader;
// pub mod writer;

pub fn write_remaining<'a, S, D, T, const C: usize>(source: &S, destination: &mut D) -> usize
where
    T: dasp::Sample,
    S: Buffer<C, Sample = T>,
    D: BufferMut<C, Sample = T>,
{
    let mut written = 0;
    for (src_frame, mut dst_frame) in source.iter_frames().zip(destination.iter_frames_mut()) {
        for (s, d) in src_frame.iter_samples().zip(dst_frame.iter_samples_mut()) {
            *d = *s;
        }
        written += 1;
    }
    written
}
