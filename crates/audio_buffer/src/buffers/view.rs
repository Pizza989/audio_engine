use crate::core::axis::{BufferAxis, BufferAxisMut};

pub trait Index<I> {
    type Output;

    fn get_indexed(&self, index: I) -> Option<&Self::Output>;
}

pub trait IndexMut<I>: Index<I> {
    fn get_indexed_mut(&mut self, index: I) -> Option<&mut Self::Output>;
}

/// # Safety
/// Implementors must guarantee that the mapping function never maps two different
/// input indices to the same output index. Violating this invariant can lead to
/// mutable aliasing and undefined behavior.
pub struct InjectiveFn<I, J>(pub Box<dyn Fn(I) -> J>);

impl<I, J> InjectiveFn<I, J> {
    fn call(&self, index: I) -> J {
        (self.0)(index)
    }
}

// A view over indexable data that transforms indices through a mapping function
pub struct View<'a, D, I, J>
where
    D: Index<J>,
{
    data: &'a D,
    mapper: Box<dyn Fn(I) -> J>,
    _phantom: std::marker::PhantomData<(I, J)>,
}

impl<'a, D, I, J> View<'a, D, I, J>
where
    D: Index<J>,
{
    pub fn new(data: &'a D, mapper: Box<dyn Fn(I) -> J>) -> Self {
        Self {
            data,
            mapper,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a value by applying the index transformation
    pub fn get(&self, index: I) -> Option<&D::Output> {
        let mapped_index = (self.mapper)(index);
        self.data.get_indexed(mapped_index)
    }
}

impl<'a, D> View<'a, D, usize, usize>
where
    D: Index<usize>,
{
    pub fn with_stride(data: &'a D, num_channels: usize, channel_index: usize) -> Self {
        Self {
            data,
            mapper: Box::new(move |sample_index: usize| {
                sample_index * num_channels + channel_index
            }),
            _phantom: std::marker::PhantomData,
        }
    }
}

// A mutable view over indexable data that transforms indices through a mapping function
pub struct MutableView<'a, D, I, J>
where
    D: IndexMut<J>,
{
    data: *mut D,
    mapper: InjectiveFn<I, J>,
    _phantom: std::marker::PhantomData<(&'a mut D, I, J)>, // SAFETY: keeps the lifetime of `data`
}

impl<'a, D, I, J> MutableView<'a, D, I, J>
where
    D: IndexMut<J>,
{
    /// Create a `MutableView` from a raw pointer and a mapping function.
    ///
    /// # SAFETY
    /// `mapper` has to be an injective function as otherwise aliasing will occur
    pub unsafe fn from_raw(data: *mut D, mapper: InjectiveFn<I, J>) -> Self {
        Self {
            data,
            mapper,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a value by applying the index transformation
    pub fn get(&self, index: I) -> Option<&D::Output> {
        let mapped_index = self.mapper.call(index);

        // SAFETY:
        // 1. lifetime: self is valid because its lifetime is kept in `_phantom`
        unsafe { (*self.data).get_indexed(mapped_index) }
    }

    /// Get a mutable reference by applying the index transformation
    pub fn get_mut(&mut self, index: I) -> Option<&mut D::Output> {
        let mapped_index = self.mapper.call(index);

        // SAFETY:
        // 1. lifetime: self is valid because its lifetime is kept in `_phantom`
        // 2. aliasing: user guarrantees aliasing doesn't occur by passing a mapping function that is injective
        unsafe { (*self.data).get_indexed_mut(mapped_index) }
    }

    /// Set a value by applying the index transformation
    pub fn set(&mut self, index: I, value: D::Output) -> Option<D::Output>
    where
        D::Output: Sized,
    {
        let slot = self.get_mut(index)?;
        Some(std::mem::replace(slot, value))
    }
}

impl<'a, D, J> BufferAxis<D::Output> for View<'a, D, usize, J>
where
    D: Index<J>,
{
    fn get_sample(&self, index: usize) -> Option<&D::Output> {
        self.get(index)
    }
}

impl<'a, D, J> BufferAxis<D::Output> for MutableView<'a, D, usize, J>
where
    D: IndexMut<J>,
{
    fn get_sample(&self, index: usize) -> Option<&D::Output> {
        self.get(index)
    }
}

impl<'a, D, J> BufferAxisMut<'a, D::Output> for MutableView<'a, D, usize, J>
where
    D: IndexMut<J>,
{
    fn get_sample_mut(&mut self, index: usize) -> Option<&mut D::Output> {
        self.get_mut(index)
    }
}

pub struct MutableViewIterMut<'view, D, J>
where
    D: IndexMut<J>,
{
    view: &'view mut MutableView<'view, D, usize, J>,
    index: usize,
}

impl<'a, D, J> Iterator for MutableViewIterMut<'a, D, J>
where
    D: IndexMut<J>,
{
    type Item = &'a mut D::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        let sample = self.view.get_sample_mut(index)?;

        // SAFETY:
        // Mutable Aliasing
        // 1. Each index passed to get_sample_mut is different between iterations
        // 2. The mapping function used is injective
        // 3. Therefore sample is always a unique reference
        //
        // Lifetime
        // 1. The Iterator holds a mutable reference with lifetime 'a to the MutableView
        // 2. The Iterator consumes itsself so the returned references can't dangle
        Some(unsafe { &mut *(sample as *mut D::Output) })
    }
}
