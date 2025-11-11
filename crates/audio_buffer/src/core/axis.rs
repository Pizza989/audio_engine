use std::{convert::AsRef, marker::PhantomData};

pub trait BufferAxis<T> {
    fn get_sample(&self, index: usize) -> Option<&T>;
    fn iter_samples(&self) -> BufferAxisIter<'_, Self, T>
    where
        Self: Sized,
    {
        BufferAxisIter {
            axis: self,
            index: 0,
            _marker: PhantomData,
        }
    }
}

pub struct BufferAxisIter<'a, A, T>
where
    A: BufferAxis<T>,
{
    axis: &'a A,
    index: usize,
    _marker: PhantomData<T>,
}

impl<'a, A, T: 'a> Iterator for BufferAxisIter<'a, A, T>
where
    A: BufferAxis<T>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.axis.get_sample(index)
    }
}

pub trait BufferAxisMut<'a, T>: BufferAxis<T> {
    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T>;
    fn map_samples_mut<F, R>(&mut self, mut f: F, offset: Option<usize>)
    where
        F: for<'sample> FnMut(&'sample mut T, usize) -> Option<R>,
    {
        let mut index = offset.unwrap_or(0);
        while let Some(sample) = self.get_sample_mut(index) {
            match f(sample, index) {
                Some(_) => {
                    index += 1;
                }
                None => break,
            };
        }
    }
}

impl<T, U> BufferAxis<T> for U
where
    U: AsRef<[T]>,
{
    fn get_sample(&self, index: usize) -> Option<&T> {
        self.as_ref().get(index)
    }
}

impl<'a, T, U> BufferAxisMut<'a, T> for U
where
    U: AsMut<[T]> + AsRef<[T]>,
{
    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_mut().get_mut(index)
    }
}
