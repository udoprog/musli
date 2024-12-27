use super::RawVec;

/// An allocator that can be used in combination with a context.
pub trait Allocator {
    /// The type of an allocated buffer.
    type RawVec<'this, T>: RawVec<T>
    where
        Self: 'this,
        T: 'this;

    /// Construct an empty uninitialized raw vector with an alignment matching
    /// that of `T` that is associated with this allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{Allocator, RawVec};
    ///
    /// let values: [u32; 4] = [1, 2, 3, 4];
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.new_raw_vec::<u32>();
    ///     let mut len = 0;
    ///
    ///     for value in values {
    ///         if !buf.resize(len, 1) {
    ///             panic!("Allocation failed");
    ///         }
    ///
    ///         // SAFETY: We've just resized the above buffer.
    ///         unsafe {
    ///             buf.as_mut_ptr().add(len).write(value);
    ///         }
    ///
    ///         len += 1;
    ///     }
    ///
    ///     // SAFETY: Slice does not outlive the buffer it references.
    ///     let bytes = unsafe { core::slice::from_raw_parts(buf.as_ptr(), len) };
    ///     assert_eq!(bytes, values);
    /// });
    /// ```
    fn new_raw_vec<'a, T>(&'a self) -> Self::RawVec<'a, T>
    where
        T: 'a;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type RawVec<'this, T>
        = A::RawVec<'this, T>
    where
        Self: 'this,
        T: 'this;

    #[inline(always)]
    fn new_raw_vec<'a, T>(&'a self) -> Self::RawVec<'a, T>
    where
        T: 'a,
    {
        (*self).new_raw_vec::<T>()
    }
}
