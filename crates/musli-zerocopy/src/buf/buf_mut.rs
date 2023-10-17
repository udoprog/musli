use core::marker::PhantomData;
use core::mem::{replace, size_of, size_of_val};

use crate::buf::Padder;
use crate::traits::ZeroCopy;

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
pub struct BufMut<'a> {
    start: *mut u8,
    _marker: PhantomData<&'a [u8]>,
}

impl<'a> BufMut<'a> {
    pub(crate) fn new(start: *mut u8) -> Self {
        Self {
            start,
            _marker: PhantomData,
        }
    }
}

impl<'a> BufMut<'a> {
    /// Store the raw bytes associated with `[T]` into the buffer and advance
    /// its position by `size_of::<T>() * bytes.len()`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer has the capacity for
    /// `bytes.len()` and that the value being stored is not padded as per
    /// `ZeroCopy::PADDED`.
    ///
    /// Also see the [type level safety documentation][#safety]
    pub unsafe fn store_bytes<T>(&mut self, bytes: &[T])
    where
        T: ZeroCopy,
    {
        self.start
            .copy_from_nonoverlapping(bytes.as_ptr().cast(), size_of_val(bytes));
        self.start = self.start.wrapping_add(size_of_val(bytes));
    }

    /// Store the raw bytes associated with `*const T` into the buffer and
    /// advance its position by `size_of::<T>()`.
    ///
    /// This does not require `T` to be aligned.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any store call only includes data up-to the
    /// size of `Self`.
    ///
    /// Also see the [type level safety documentation][#safety]
    pub unsafe fn store_unaligned<T>(&mut self, value: *const T)
    where
        T: ZeroCopy,
    {
        self.start
            .copy_from_nonoverlapping(value.cast(), size_of::<T>());
        self.start = self.start.wrapping_add(size_of::<T>());
    }

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
    /// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
    /// use musli_zerocopy::buf::BufMut;
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
    /// // You're responsible for reserving more elements when using this API.
    /// buf.reserve(size_of::<ZeroPadded>() * 10);
    ///
    /// unsafe {
    ///     let mut buf_mut = buf.as_buf_mut();
    ///
    ///     for _ in 0..10 {
    ///         // SAFETY: We do not pad beyond known fields and are
    ///         // making sure to initialize all of the buffer.
    ///         let mut w = buf_mut.store_struct(&padded);
    ///         w.pad(&padded.a);
    ///         w.pad(&padded.b);
    ///         w.pad(&padded.c);
    ///         w.pad(&padded.d);
    ///         w.remaining();
    ///
    ///         padded.a += 1;
    ///     }
    ///
    ///     buf.advance(size_of::<ZeroPadded>() * 10);
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
    pub unsafe fn store_struct<T>(&mut self, value: *const T) -> Padder<'_, T>
    where
        T: ZeroCopy,
    {
        let end = self.start.wrapping_add(size_of::<T>());

        self.start
            .copy_from_nonoverlapping(value.cast(), size_of::<T>());

        let start = replace(&mut self.start, end);
        Padder::new(start)
    }
}
