/// A raw buffer allocated from a [`Context`].
///
/// [`Context`]: crate::Context
pub trait Buf<T> {
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
        B: Buf<T>;
}
