use std::marker::PhantomData;

/// A strided slice that provides safe access to non-contiguous data
pub struct StridedSlice<'a, T> {
    ptr: *const T,
    len: usize,
    stride: usize,
    _life: PhantomData<&'a [T]>,
}

/// A mutable strided slice
pub struct StridedSliceMut<'a, T> {
    ptr: *mut T,
    len: usize,
    stride: usize,
    _life: PhantomData<&'a mut [T]>,
}

impl<'a, T> StridedSlice<'a, T> {
    /// Create a new strided slice
    ///
    /// # Safety
    /// - `data` must be valid for reads for the entire lifetime 'a
    /// - `start + (len - 1) * stride` must be within bounds of the data
    /// - All elements accessed via this stride must be properly initialized
    pub(crate) unsafe fn new(data: &'a [T], start: usize, len: usize, stride: usize) -> Self {
        Self {
            ptr: unsafe { data.as_ptr().add(start) },
            len,
            stride,
            _life: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index * self.stride)) }
        } else {
            None
        }
    }

    pub fn iter(&self) -> StridedIter<'a, T> {
        StridedIter {
            ptr: self.ptr,
            len: self.len,
            stride: self.stride,
            index: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> StridedSliceMut<'a, T> {
    /// Create a new mutable strided slice
    ///
    /// # Safety
    /// Same requirements as StridedSlice::new, but for mutable access
    pub(crate) unsafe fn new(data: &'a mut [T], start: usize, len: usize, stride: usize) -> Self {
        Self {
            ptr: unsafe { data.as_mut_ptr().add(start) },
            len,
            stride,
            _life: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index * self.stride)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index * self.stride)) }
        } else {
            None
        }
    }

    pub fn iter(&self) -> StridedIter<'_, T> {
        StridedIter {
            ptr: self.ptr,
            len: self.len,
            stride: self.stride,
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> StridedIterMut<'_, T> {
        StridedIterMut {
            ptr: self.ptr,
            len: self.len,
            stride: self.stride,
            index: 0,
            _marker: PhantomData,
        }
    }
}

/// Iterator for strided slices
pub struct StridedIter<'a, T> {
    ptr: *const T,
    len: usize,
    stride: usize,
    index: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for StridedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = unsafe { &*self.ptr.add(self.index * self.stride) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for StridedIter<'a, T> {}

/// Mutable iterator for strided slices
pub struct StridedIterMut<'a, T> {
    ptr: *mut T,
    len: usize,
    stride: usize,
    index: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for StridedIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = unsafe { &mut *self.ptr.add(self.index * self.stride) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for StridedIterMut<'a, T> {}
