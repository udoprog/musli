use crate::buf::Padder;
use crate::traits::ZeroCopy;

mod sealed {
    #[cfg(feature = "alloc")]
    use crate::pointer::Size;

    pub trait Sealed {}

    #[cfg(feature = "alloc")]
    impl<O: Size> Sealed for crate::buf::OwnedBuf<O> {}
    impl<B: ?Sized> Sealed for &mut B where B: Sealed {}
    impl Sealed for crate::buf::RawBufMut {}
}

/// A mutable buffer to store zero copy types to.
///
/// This is implemented by [`OwnedBuf`].
///
/// # Safety
///
/// Every store function is unsafe, because every buffer pre-allocates space for
/// the type being stored and calling `store*` incorrectly would result in
/// writing out-of-bound.
///
/// [`OwnedBuf`]: crate::OwnedBuf
pub trait BufMut: self::sealed::Sealed {
    /// Extend the current buffer from the given slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer has the capacity for
    /// `bytes.len()` and that the value being stored is not padded as per
    /// `ZeroCopy::PADDED`.
    ///
    /// Also see the [type level safety documentation][#safety]
    unsafe fn store_bytes<T>(&mut self, bytes: &[T])
    where
        T: ZeroCopy;

    /// Store the exact bits of the given ZeroCopy type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any store call only includes data up-to the
    /// size of `Self`.
    ///
    /// Also see the [type level safety documentation][#safety]
    unsafe fn store_bits<T>(&mut self, value: *const T)
    where
        T: ZeroCopy;

    /// Write the given zero copy type to the buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any store call only includes data up-to the
    /// size of `Self`.
    ///
    /// Also see the [type level safety documentation][#safety]
    unsafe fn store<T>(&mut self, value: &T)
    where
        T: ZeroCopy;

    /// Store the given struct and return a [`Padder`] to initialize the
    /// any padding in the type written.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any store call only includes data up-to the
    /// size of `Self`.
    ///
    /// The caller must also ensure to [`pad()`] the output correctly according
    /// to the type being encoded, or else the aligned buffer will end up with
    /// uninitialized bytes.
    ///
    /// Also see the [type level safety documentation][#safety]
    ///
    /// [`pad()`]: Padder::pad
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem::size_of;
    ///
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
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
    /// let mut buf = OwnedBuf::new();
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
    ///         w.pad(&padded.a);
    ///         w.pad(&padded.b);
    ///         w.pad(&padded.c);
    ///         w.pad(&padded.d);
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
    /// let buf = buf.into_aligned();
    ///
    /// padded.a = 0;
    /// assert_eq!(buf.load(reference)?, &padded);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    unsafe fn store_struct<T>(&mut self, value: *const T) -> Padder<'_, T>
    where
        T: ZeroCopy;

    /// Store an array immediately in the buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any store call only includes data up-to the
    /// size of `Self`.
    ///
    /// Also see the [type level safety documentation][#safety]
    unsafe fn store_array<T>(&mut self, values: &[T])
    where
        T: ZeroCopy;
}
