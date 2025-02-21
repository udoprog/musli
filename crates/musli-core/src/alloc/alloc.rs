use core::ptr::NonNull;

use super::AllocError;

/// A value allocated through [`Allocator::alloc`].
///
/// [`Allocator::alloc`]: super::Allocator::alloc
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
///
/// Example of a correctly managed slice allocation:
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
pub trait Alloc<T>
where
    Self: Sized,
{
    /// Get a pointer into the allocation.
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
    fn as_ptr(&self) -> *const T;

    /// Get a mutable pointer into the allocation.
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
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Returns the capacity of the buffer.
    fn capacity(&self) -> usize;

    /// Resize the buffer to fit at least additional elements.
    ///
    /// Returns the new capacity of the buffer.
    fn resize(&mut self, len: usize, additional: usize) -> Result<(), AllocError>;

    /// Try to merge one buffer with another.
    ///
    /// The two length parameters refers to the initialized length of the two
    /// buffers.
    ///
    /// If this returns `Err(B)` if merging was not possible.
    ///
    /// Returns the capacity of the newly merged buffer.
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: Alloc<T>;

    /// Convert an allocation into an `alloc` allocation if it is possible with
    /// the allocator that constructed the allocation.
    ///
    /// # Safety
    ///
    /// The implementor must ensure that the returned pointer is valid and
    /// originates from the system allocator through the `alloc` module.
    #[inline]
    unsafe fn as_non_null_std(&mut self) -> Option<(NonNull<u8>, usize)> {
        None
    }
}
