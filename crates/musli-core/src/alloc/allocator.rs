use super::{Alloc, AllocError};

/// An allocator that can be used in combination with a context.
///
/// # Safety
///
/// Setting `IS_SYSTEM` to `true` has safety implications, since it determines
/// whether the allocation can be safely converted into a standard container or
/// not.
pub unsafe trait Allocator: Copy {
    /// Whether the allocations returned by this allocatore is backed by the
    /// system allocator or not.
    ///
    /// # Safety
    ///
    /// Setting this to `true` has safety implications, since it determines
    /// whether the allocation can be safely converted into a standard container
    /// or not.
    const IS_SYSTEM: bool;

    /// A raw allocation from the allocator.
    type Alloc<T>: Alloc<T>;

    /// Construct an empty uninitialized raw vector with an alignment matching
    /// that of `T` that is associated with this allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::slice;
    /// use musli::alloc::{Alloc, AllocError, Allocator};
    ///
    /// let values: [u32; 4] = [1, 2, 3, 4];
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc_empty::<u32>();
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
    ///     let bytes = unsafe { slice::from_raw_parts(buf.as_ptr(), len) };
    ///     assert_eq!(bytes, values);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    fn alloc_empty<T>(self) -> Self::Alloc<T>;

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
}
