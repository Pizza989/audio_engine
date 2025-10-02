# Context

I'm developping a rust crate that abstracts audio buffers. A key challange are non continuous slices.
My solution to those is the MutableView, it's supposed to remap a continuous index to a non continuous
one using a mapping function.
A core part of the crate is the Buffer trait that i haven't included. It allows indexing a buffer
along axis that are represented by the BufferAxis trait that i have included.
The way this works is that a frame of the buffer implements BufferAxis. For non continuous Frames the
concrete type would be View or MutableView.
Therefore i need to implement BufferAxisMut for MutableView:

```rust
impl<'a, D, M, J> BufferAxisMut<D::Output> for MutableView<'a, D, M, usize, J>
where
    D: IndexMut<J>,
    M: InjectiveMapper<usize, J>,
    D::Output: 'a,
{
    fn get_sample_mut(&mut self, index: usize) -> Option<&mut D::Output> {
        self.get_mut(index)
    }

    fn iter_samples_mut<'this>(&'this mut self) -> impl Iterator<Item = &'this mut D::Output>
    where
        D::Output: 'this,
    {
        MutableViewIterMut { // lifetime may not live long enough
            view: self,      // consider adding the following bound: `'this: 'a`
            index: 0,
        }
    }
}
```

As I have annotated in the code block I get a compiler error, saying that:

```
lifetime may not live long enough
consider adding the following bound: `'this: 'a`
```


# The Problem

Firstly the compiler suggest adding a bound however that is not possible because it is not in
the trait's signature. And it can't be in it either because the lifetime 'a is specifcic to
the MutableView struct.
Secondly the bound `'this: 'a` means that 'this must outlive 'a. But isn't that obvious because 'this is the
lifetime of an instance of the struct and the struct has the lifetime 'a?

Regardless of this suggestion though, I can't apply it anyways, so it seems like the pattern how I implemented
this doesn't work. Is that the case?

I'd be very happy if you can solve this but I would also appreciate knowing if this is a common pattern or
if there are common patterns for doing what I want. Or even a resource that explains parts of my questions.

# Other Relevant Types

```rust
pub trait Index<I> {
    type Output;

    fn get(&self, index: I) -> Option<&Self::Output>;
}

pub trait IndexMut<I>: Index<I> {
    fn get_mut(&mut self, index: I) -> Option<&mut Self::Output>;
}

/// Marker trait indicating that a mapping function is injective (one-to-one).
///
/// # Safety
/// Implementors must guarantee that the mapping function never maps two different
/// input indices to the same output index. Violating this invariant can lead to
/// mutable aliasing and undefined behavior.
///
/// A function `f` is injective if: for all `a != b`, `f(a) != f(b)`
pub unsafe trait InjectiveMapper<I, J> {
    fn map(&self, index: I) -> J;
}

{...}

// A mutable view over indexable data that transforms indices through a mapping function
pub struct MutableView<'a, D, M, I, J>
where
    D: IndexMut<J>,
    M: InjectiveMapper<I, J>,
{
    data: &'a mut D,
    mapper: M,
    _phantom: std::marker::PhantomData<(I, J)>,
}

impl<'a, D, M, I, J> MutableView<'a, D, M, I, J>
where
    D: IndexMut<J>,
    M: InjectiveMapper<I, J>,
{
    pub fn new(data: &'a mut D, mapper: M) -> Self {
        Self {
            data,
            mapper,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a value by applying the index transformation
    pub fn get(&self, index: I) -> Option<&D::Output> {
        let mapped_index = self.mapper.map(index);
        self.data.get(mapped_index)
    }

    /// Get a mutable reference by applying the index transformation
    pub fn get_mut(&mut self, index: I) -> Option<&mut D::Output> {
        let mapped_index = self.mapper.map(index);
        self.data.get_mut(mapped_index)
    }

    /// Set a value by applying the index transformation
    pub fn set(&mut self, index: I, value: D::Output) -> Option<D::Output>
    where
        D::Output: Sized,
    {
        let mapped_index = self.mapper.map(index);
        let slot = self.data.get_mut(mapped_index)?;
        Some(std::mem::replace(slot, value))
    }
}

{...}

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

{...}

pub trait BufferAxisMut<T>: BufferAxis<T> {
    fn get_sample_mut(&mut self, index: usize) -> Option<&mut T>;
    fn iter_samples_mut<'this>(&'this mut self) -> impl Iterator<Item = &'this mut T>
    where
        T: 'this;
}
```
