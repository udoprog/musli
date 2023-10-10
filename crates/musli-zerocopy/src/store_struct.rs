use crate::error::Error;
use crate::r#ref::Ref;
use crate::zero_copy::ZeroCopy;

/// A writer as returned from [`BufMut::store_struct`].
///
/// [`BufMut::store_struct`]: crate::buf_mut::BufMut::store_struct
pub trait StoreStruct<T> {
    /// Pad around the given field with zeros.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// requirements imposed by [`finish()`].
    ///
    /// [`finish()`]: Self::finish
    fn pad<F>(&mut self)
    where
        F: ZeroCopy;

    /// Finish writing the current buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that they've called [`pad`] in order for every
    /// field in a struct being serialized. Otherwise we might not have written
    /// the necessary padding, and the [`AlignedBuf`] we're writing to might
    /// contain uninitialized data in the form of uninitialized padding.
    ///
    /// [`pad`]: Self::pad
    unsafe fn finish(self) -> Result<Ref<T>, Error>;
}
