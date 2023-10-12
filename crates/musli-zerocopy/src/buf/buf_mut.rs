use crate::buf::StructPadder;
use crate::error::Error;
use crate::pointer::Size;
use crate::traits::ZeroCopy;

mod sealed {
    #[cfg(feature = "alloc")]
    use crate::pointer::Size;

    pub trait Sealed {}

    #[cfg(feature = "alloc")]
    impl<O: Size> Sealed for crate::buf::AlignedBuf<O> {}
    impl<B: ?Sized> Sealed for &mut B where B: Sealed {}
}

/// A mutable buffer to store zero copy types to.
///
/// This is implemented by [`AlignedBuf`].
///
/// [`AlignedBuf`]: crate::AlignedBuf
pub trait BufMut: self::sealed::Sealed {
    /// Target size buffer is configured to use.
    type Size: Size;

    /// Extend the current buffer from the given slice.
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Write the given zero copy type to the buffer.
    fn store<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy;

    /// Store the given struct and return a [`StructPadder`] to initialize the
    /// any padding in the type written.
    ///
    /// # Safety
    ///
    /// The caller must ensure to [`pad()`] the output correctly according to
    /// the type being encoded, or else the aligned buffer will end up with
    /// uninitialized bytes.
    ///
    /// [`pad()`]: StructPadder::pad
    ///
    /// # Examples
    ///
    /// ```
    /// use core::mem::size_of;
    ///
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::buf::BufMut;
    /// use musli_zerocopy::pointer::Ref;
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
    /// let mut padded = ZeroPadded {
    ///     a: 0,
    ///     b: 0x0203_0405_0607_0809u64.to_be(),
    ///     c: 0x0a0bu16.to_be(),
    ///     d: 0x0c0d_0e0fu32.to_be(),
    /// };
    ///
    /// let reference = Ref::<ZeroPadded>::new(buf.next_offset::<ZeroPadded>());
    ///
    /// for _ in 0..10 {
    ///     // SAFETY: We do not pad beyond known fields and are
    ///     // making sure to initialize all of the buffer.
    ///     unsafe {
    ///         let mut w = buf.store_struct(&padded);
    ///         w.pad::<u8>();
    ///         w.pad::<u64>();
    ///         w.pad::<u16>();
    ///         w.pad::<u32>();
    ///         w.end();
    ///     };
    ///
    ///     padded.a += 1;
    /// }
    ///
    /// for (index, chunk) in buf.as_slice().chunks_exact(size_of::<ZeroPadded>()).enumerate() {
    ///     // Note: The bytes are explicitly convert to big-endian encoding above.
    ///     assert_eq!(chunk, &[index as u8, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 12, 13, 14, 15]);
    /// }
    ///
    /// let buf = buf.as_aligned();
    ///
    /// padded.a = 0;
    /// assert_eq!(buf.load(reference)?, &padded);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    unsafe fn store_struct<T>(&mut self, value: &T) -> StructPadder<'_, T>
    where
        T: ZeroCopy;

    /// Store the contents of a `ZeroCopy` array.
    fn store_array<T, const N: usize>(&mut self, array: &[T; N]) -> Result<(), Error>
    where
        T: ZeroCopy;
}

impl<B: ?Sized> BufMut for &mut B
where
    B: BufMut,
{
    type Size = B::Size;

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
    unsafe fn store_struct<T>(&mut self, value: &T) -> StructPadder<'_, T>
    where
        T: ZeroCopy,
    {
        (**self).store_struct(value)
    }

    #[inline]
    fn store_array<T, const N: usize>(&mut self, array: &[T; N]) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        (**self).store_array(array)
    }
}
