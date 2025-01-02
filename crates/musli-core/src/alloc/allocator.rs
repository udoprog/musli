use super::{Alloc, AllocError, AllocSlice};

/// An allocator that can be used in combination with a context.
pub trait Allocator {
    /// A raw allocation from the allocator.
    type Alloc<T>: Alloc<T>;

    /// The type of an allocated buffer.
    type AllocSlice<T>: AllocSlice<T>;

    /// Construct an empty uninitialized raw allocation with an alignment
    /// matching that of `T` that is associated with this allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Allocator, Alloc};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc(10u32)?;
    ///
    ///     unsafe {
    ///         buf.as_mut_ptr().write(20u32);
    ///         assert_eq!(buf.as_ptr().read(), 20u32);
    ///     }
    ///
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    fn alloc<T>(self, value: T) -> Result<Self::Alloc<T>, AllocError>;

    /// Construct an empty uninitialized raw vector with an alignment matching
    /// that of `T` that is associated with this allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Allocator, AllocSlice};
    ///
    /// let values: [u32; 4] = [1, 2, 3, 4];
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc_slice::<u32>();
    ///     let mut len = 0;
    ///
    ///     for value in values {
    ///         buf.resize(len, 1)?;
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
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    fn alloc_slice<T>(self) -> Self::AllocSlice<T>;
}
