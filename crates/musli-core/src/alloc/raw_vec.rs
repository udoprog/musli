/// A raw buffer allocated through an [`Allocator`].
///
/// [`Allocator`]: super::Allocator
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
pub trait RawVec<T> {
    /// Resize the buffer.
    fn resize(&mut self, len: usize, additional: usize) -> bool;

    /// Get a pointer into the buffer.
    fn as_ptr(&self) -> *const T;

    /// Get a mutable pointer into the buffer.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Try to merge one buffer with another.
    ///
    /// The two length parameters refers to the initialized length of the two
    /// buffers.
    ///
    /// If this returns `Err(B)` if merging was not possible.
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: RawVec<T>;
}
