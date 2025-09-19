use dasp::Sample;

use crate::core::{
    Buffer,
    io::{Stream, StreamMut},
};

/// Writer wraps any *mutable* buffer providing mutable frame iterators.
/// Keeps a cursor and implements `Stream` + `StreamMut`.
pub struct Writer<'a, B, T, const C: usize>
where
    T: Sample,
    B: Buffer<C, Sample = T> + ?Sized,
{
    buffer: &'a mut B,
    pos: usize,
}

impl<'a, B, T, const C: usize> Writer<'a, B, T, C>
where
    T: Sample,
    B: Buffer<C, Sample = T> + ?Sized,
{
    pub fn new(buffer: &'a mut B) -> Self {
        Self { buffer, pos: 0 }
    }

    /// Get a mutable reference to the underlying buffer
    pub fn buffer_mut(&mut self) -> &mut B {
        self.buffer
    }
}

impl<'a, B, T, const C: usize> Stream<T, C> for Writer<'a, B, T, C>
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

    fn read_block(&mut self, out: &mut [[T; C]]) -> Result<usize, ()> {
        // We can provide read access via the mutable buffer as well:
        if self.pos >= self.frames() || out.is_empty() {
            return Err(());
        }

        // Make an immutable iterator by temporarily borrowing as immutable if possible.
        // Since `iter_frames` takes &self, and we have &mut B, we can call it.
        let mut it = self.buffer.iter_frames();
        for _ in 0..self.pos {
            if it.next().is_none() {
                return Err(());
            }
        }

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

impl<'a, B, T, const C: usize> StreamMut<T, C> for Writer<'a, B, T, C>
where
    T: Sample + Copy,
    B: Buffer<C, Sample = T> + ?Sized,
{
    fn write_block(&mut self, input: &[[T; C]]) -> Result<usize, ()> {
        if input.is_empty() || self.pos >= self.frames() {
            return Err(());
        }

        let mut it = self.buffer.iter_frames_mut();
        for _ in 0..self.pos {
            if it.next().is_none() {
                return Err(());
            }
        }

        let mut i = 0usize;
        while i < input.len() {
            match it.next() {
                Some(dst) => {
                    *dst = input[i];
                    i += 1;
                    self.pos += 1;
                }
                None => break,
            }
        }
        Ok(i)
    }
}
