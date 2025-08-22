use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

use super::DEFAULT_ARRAY_BUFFER;

/// An array that can conveniently be used as a buffer, by default this is
/// [`DEFAULT_ARRAY_BUFFER`] bytes large.
///
/// This is aligned to 8 bytes, since that's an alignment which works with many
/// common Rust types.
///
/// See the [module level documentation][super] for more information.
#[repr(align(8))]
pub struct ArrayBuffer<const N: usize = DEFAULT_ARRAY_BUFFER> {
    data: [MaybeUninit<u8>; N],
}

impl ArrayBuffer {
    /// Construct a new buffer with the default size of
    /// [`DEFAULT_ARRAY_BUFFER`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::ArrayBuffer;
    ///
    /// let buffer = ArrayBuffer::new();
    /// assert_eq!(buffer.len(), 4096);
    /// ```
    pub const fn new() -> Self {
        Self::with_size()
    }
}

impl Default for ArrayBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ArrayBuffer<N> {
    /// Construct a new buffer with a custom size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::ArrayBuffer;
    ///
    /// let buffer = ArrayBuffer::<1024>::with_size();
    /// assert_eq!(buffer.len(), 1024);
    /// ```
    pub const fn with_size() -> Self {
        Self {
            // SAFETY: This is safe to initialize, since it's just an array of
            // contiguous uninitialized memory.
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<const N: usize> Deref for ArrayBuffer<N> {
    type Target = [MaybeUninit<u8>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize> DerefMut for ArrayBuffer<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
