use crate::Buf;

/// An allocator that can be used in combination with a context.
///
/// See the [musli-allocator] crate for examples.
///
/// [musli-allocator]: https://crates.io/crates/musli-allocator
pub trait Allocator {
    /// The type of an allocated buffer.
    type Buf<'this>: Buf
    where
        Self: 'this;

    /// Allocate an empty, uninitialized buffer with an alignment matching that
    /// of `T`.
    ///
    /// Calling this method returns `None` if the allocation failed.
    fn alloc_aligned<T>(&self) -> Option<Self::Buf<'_>>;

    /// Allocate an empty, uninitialized buffer with an alignment of 1.
    ///
    /// Calling this method returns `None` if the allocation failed.
    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc_aligned::<u8>()
    }
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc_aligned<T>(&self) -> Option<Self::Buf<'_>> {
        (*self).alloc_aligned::<T>()
    }

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        (*self).alloc()
    }
}
