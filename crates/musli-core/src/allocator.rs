use crate::Buf;

/// An allocator that can be used in combination with a context.
///
/// See the [musli-allocator] crate for examples.
///
/// [musli-allocator]: https://crates.io/crates/musli-allocator
pub trait Allocator {
    /// The type of an allocated buffer.
    type Buf<'this, T>: Buf<Item = T>
    where
        Self: 'this,
        T: 'static;

    /// Allocate an empty, uninitialized buffer with an alignment matching that
    /// of `T`.
    ///
    /// Calling this method returns `None` if the allocation failed.
    fn alloc<T>(&self) -> Option<Self::Buf<'_, T>>
    where
        T: 'static;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf<'this, T> = A::Buf<'this, T> where Self: 'this, T: 'static;

    #[inline(always)]
    fn alloc<T>(&self) -> Option<Self::Buf<'_, T>>
    where
        T: 'static,
    {
        (*self).alloc::<T>()
    }
}
