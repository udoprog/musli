use crate::buf::Buf;

/// An allocator that can be used in combination with a context.
pub trait Allocator {
    /// The type of an allocated buffer.
    type Buf<'this, T>: Buf<T>
    where
        Self: 'this,
        T: 'this;

    /// Allocate an empty, uninitialized buffer with an alignment matching that
    /// of `T`.
    ///
    /// Calling this method returns `None` if the allocation failed.
    fn alloc<'a, T>(&'a self) -> Self::Buf<'a, T>
    where
        T: 'a;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf<'this, T> = A::Buf<'this, T> where Self: 'this, T: 'this;

    #[inline(always)]
    fn alloc<'a, T>(&'a self) -> Self::Buf<'a, T>
    where
        T: 'a,
    {
        (*self).alloc::<T>()
    }
}
