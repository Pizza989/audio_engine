use dasp::Sample;

use crate::core::{Buffer, io::Stream};

/// Reader wraps any buffer that can provide an iterator of frames.
/// It keeps a cursor and implements `Stream`.
pub struct Reader<'a, B, const C: usize>
where
    B: Buffer<C> + ?Sized,
{
    buffer: &'a B,
    pos: usize,
}

impl<'a, B, const C: usize> Reader<'a, B, T, C>
where
    T: Sample,
    B: Buffer<C, Sample = T> + ?Sized,
{
    pub fn new(buffer: &'a B) -> Self {
        Self { buffer, pos: 0 }
    }

    /// Get a reference to the underlying buffer
    pub fn buffer(&self) -> &B {
        self.buffer
    }
}

impl<'a, B, T, const C: usize> Stream<T, C> for Reader<'a, B, T, C>
where
    T: Sample + Copy,
    B: Buffer<C, Sample = T> + ?Sized,
{
    fn frames(&self) -> usize {
        self.buffer.frames()
    }

    fn position(&self) -> usize {
        self.pos
    }

    // TODO: implement Err Variants
    fn set_position(&mut self, pos: usize) -> Result<(), ()> {
        if pos < self.frames() {
            self.pos = pos;
            Ok(())
        } else {
            Err(())
        }
    }

    // TODO: implement Err Variants
    fn read_block(&mut self, out: &mut [[T; C]]) -> Result<usize, ()> {
        if self.pos >= self.frames() || out.is_empty() {
            return Err(());
        }

        // Create an iterator and advance it to current position.
        let mut it = self.buffer.iter_frames();
        for _ in 0..self.pos {
            // Safe: iterator returns Option; ignore advancement results
            if it.next().is_none() {
                return Err(());
            }
        }

        // Fill `out` with frames from iterator.
        let mut i = 0usize;
        while i < out.len() {
            match it.next() {
                Some(frame) => {
                    out[i] = frame;
                    i += 1;
                    self.pos += 1;
                }
                None => break,
            }
        }
        Ok(i)
    }
}
