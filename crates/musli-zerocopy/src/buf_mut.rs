use crate::error::Error;
use crate::store_struct::StoreStruct;
use crate::zero_copy::ZeroCopy;

/// A mutable buffer to store zero copy types to.
///
/// This is implemented by [`AlignedBuf`].
///
/// [`AlignedBuf`]: crate::AlignedBuf
pub trait BufMut {
    /// Interior mutable buffer.
    type BufMut<'a>: BufMut
    where
        Self: 'a;

    /// Extend the current buffer from the given slice.
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Write the given zero copy type to the buffer.
    fn store<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy;

    /// The length of the buffer.
    fn len(&self) -> usize;

    /// Set the length of the buffer.
    unsafe fn set_len(&mut self, len: usize);

    /// The capacity of the buffer.
    fn capacity(&self) -> usize;

    /// Pointer to base.
    fn as_ptr_mut(&mut self) -> *mut u8;

    /// Setup a writer for the given type.
    ///
    /// This API stores the type directly using an unaligned pointer store and
    /// just ensures that any padding is zeroed.
    ///
    /// # Safety
    ///
    /// While calling just this function is not unsafe, finishing writing with
    /// [`StoreStruct::finish`] is unsafe.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{ZeroCopy, AlignedBuf, BufMut};
    ///
    /// #[derive(Debug, PartialEq, Eq, ZeroCopy)]
    /// #[repr(C)]
    /// struct ZeroPadded {
    ///     a: u8,
    ///     b: u64,
    ///     c: u16,
    ///     d: u32,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let padded = ZeroPadded {
    ///     a: 0x01u8.to_be(),
    ///     b: 0x0203_0405_0607_0809u64.to_be(),
    ///     c: 0x0a0bu16.to_be(),
    ///     d: 0x0c0d_0e0fu32.to_be(),
    /// };
    ///
    /// let mut w = buf.store_struct(&padded);
    /// w.pad::<u8>();
    /// w.pad::<u64>();
    /// w.pad::<u16>();
    /// w.pad::<u32>();
    ///
    /// // SAFETY: We've asserted that the struct fields have been correctly padded.
    /// let ptr = unsafe { w.finish()? };
    ///
    /// // Note: The bytes are explicitly convert to big-endian encoding above.
    /// assert_eq!(buf.as_slice(), &[1, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 12, 13, 14, 15]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(buf.load(ptr)?, &padded);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    fn store_struct<T>(&mut self, value: &T) -> StoreStruct<Self::BufMut<'_>, T>
    where
        T: ZeroCopy;
}

impl<B: ?Sized> BufMut for &mut B
where
    B: BufMut,
{
    type BufMut<'a> = B::BufMut<'a> where Self: 'a;

    #[inline]
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        (**self).extend_from_slice(bytes)
    }

    #[inline]
    fn store<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        (**self).store(value)
    }

    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline]
    unsafe fn set_len(&mut self, len: usize) {
        (**self).set_len(len)
    }

    #[inline]
    fn capacity(&self) -> usize {
        (**self).capacity()
    }

    #[inline]
    fn as_ptr_mut(&mut self) -> *mut u8 {
        (**self).as_ptr_mut()
    }

    #[inline]
    fn store_struct<T>(&mut self, value: &T) -> StoreStruct<Self::BufMut<'_>, T>
    where
        T: ZeroCopy,
    {
        (**self).store_struct(value)
    }
}
