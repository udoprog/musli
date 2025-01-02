use super::AllocError;

/// A slice allocated through [`Allocator::alloc_slice`].
///
/// [`Allocator::alloc_slice`]: super::Allocator::alloc_slice
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
pub trait AllocSlice<T> {
    /// Get a pointer into the buffer.
    fn as_ptr(&self) -> *const T;

    /// Get a mutable pointer into the buffer.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Resize the buffer.
    fn resize(&mut self, len: usize, additional: usize) -> Result<(), AllocError>;

    /// Try to merge one buffer with another.
    ///
    /// The two length parameters refers to the initialized length of the two
    /// buffers.
    ///
    /// If this returns `Err(B)` if merging was not possible.
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: AllocSlice<T>;
}
