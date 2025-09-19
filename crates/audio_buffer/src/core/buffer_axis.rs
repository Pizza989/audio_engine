use crate::core::stride::{StridedIter, StridedIterMut, StridedSlice, StridedSliceMut};

pub trait BufferAxis<T> {
    type Iter<'this>: Iterator<Item = &'this T>
    where
        T: 'this,
        Self: 'this;

    fn samples(&self) -> usize;
    fn get_sample(&self, index: usize) -> Option<&T>;
    fn iter_samples(&self) -> Self::Iter<'_>;
}

pub trait BufferAxisMut<T>: BufferAxis<T> {
    type IterMut<'this>: Iterator<Item = &'this mut T>
    where
        T: 'this,
        Self: 'this;

    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T>;
    fn iter_samples_mut(&mut self) -> Self::IterMut<'_>;
}

impl<T> BufferAxis<T> for &[T] {
    type Iter<'this>
        = std::slice::Iter<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn samples(&self) -> usize {
        self.len()
    }

    fn get_sample(&self, index: usize) -> Option<&T> {
        self.get(index)
    }

    fn iter_samples(&self) -> Self::Iter<'_> {
        self.iter()
    }
}

impl<T> BufferAxis<T> for &mut [T] {
    type Iter<'this>
        = std::slice::Iter<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn samples(&self) -> usize {
        self.len()
    }

    fn get_sample(&self, index: usize) -> Option<&T> {
        self.get(index)
    }

    fn iter_samples(&self) -> Self::Iter<'_> {
        self.iter()
    }
}

impl<T> BufferAxisMut<T> for &mut [T] {
    type IterMut<'this>
        = std::slice::IterMut<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut(index)
    }

    fn iter_samples_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut()
    }
}

impl<T> BufferAxis<T> for StridedSlice<'_, T> {
    type Iter<'this>
        = StridedIter<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn samples(&self) -> usize {
        self.len()
    }

    fn get_sample(&self, index: usize) -> Option<&T> {
        self.get(index)
    }

    fn iter_samples(&self) -> Self::Iter<'_> {
        self.iter()
    }
}

impl<T> BufferAxis<T> for StridedSliceMut<'_, T> {
    type Iter<'this>
        = StridedIter<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn samples(&self) -> usize {
        self.len()
    }

    fn get_sample(&self, index: usize) -> Option<&T> {
        self.get(index)
    }

    fn iter_samples(&self) -> Self::Iter<'_> {
        self.iter()
    }
}

impl<T> BufferAxisMut<T> for StridedSliceMut<'_, T> {
    type IterMut<'this>
        = StridedIterMut<'this, T>
    where
        T: 'this,
        Self: 'this;

    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut(index)
    }

    fn iter_samples_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut()
    }
}
