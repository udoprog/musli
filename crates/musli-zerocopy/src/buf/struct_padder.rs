use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;

use crate::traits::ZeroCopy;

/// A struct padder as returned from [`BufMut::store_struct`].
///
/// This knows how to find and initialize padding regions in `repr(C)` types,
/// and provides a builder-like API to doing so.
///
/// [`BufMut::store_struct`]: crate::buf::BufMut::store_struct
#[must_use = "For the writer to have an effect on `AlignedBuf` you must call `StructPadder::finish`"]
pub struct StructPadder<'a, T> {
    ptr: *mut u8,
    offset: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StructPadder<'a, T>
where
    T: ZeroCopy,
{
    #[inline]
    pub(crate) fn new(ptr: *mut u8) -> Self {
        Self {
            ptr,
            offset: 0,
            _marker: PhantomData,
        }
    }

    /// Pad around the given field with zeros.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// safety requirements by [`end()`].
    ///
    /// This is typically not called directly, but rather is implemented by the
    /// [`ZeroCopy`] derive.
    ///
    /// [`end()`]: Self::end
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being padded.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::buf::BufMut;
    ///
    /// #[derive(Debug, PartialEq, Eq, ZeroCopy)]
    /// #[repr(C)]
    /// struct ZeroPadded(u8, u16);
    ///
    /// let padded = ZeroPadded(0x01u8.to_be(), 0x0203u16.to_be());
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// // SAFETY: We do not pad beyond known fields and are
    /// // making sure to initialize all of the buffer.
    /// unsafe {
    ///     let mut w = buf.store_struct(&padded);
    ///     w.pad::<u8>(&padded.0);
    ///     w.pad::<u16>(&padded.1);
    ///     w.end();
    /// }
    ///
    /// // Note: The bytes are explicitly convert to big-endian encoding above.
    /// assert_eq!(buf.as_slice(), &[1, 0, 2, 3]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub unsafe fn pad<F>(&mut self, field: *const F)
    where
        F: ZeroCopy,
    {
        let count = crate::buf::padding_to(self.offset, align_of::<F>());
        // zero out padding.
        ptr::write_bytes(self.ptr.add(self.offset), 0, count);
        self.offset = self.offset.wrapping_add(count);

        if F::PADDED {
            let mut padder = StructPadder::new(self.ptr.wrapping_add(self.offset));
            F::pad(field, &mut padder);
            padder.end();
        }

        self.offset = self.offset.wrapping_add(size_of::<F>());
    }

    /// Only pad a field where the value of the field doesn't matter.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being padded and that `F` is a primitive that does
    /// not contain any interior padding.
    #[inline]
    pub unsafe fn pad_primitive<F>(&mut self)
    where
        F: ZeroCopy,
    {
        let count = crate::buf::padding_to(self.offset, align_of::<F>());
        // zero out padding.
        ptr::write_bytes(self.ptr.add(self.offset), 0, count);
        self.offset = self.offset.wrapping_add(count.wrapping_add(size_of::<F>()));
    }

    /// Finish writing the current buffer.
    ///
    /// This is typically not called directly, but rather is implemented by the
    /// [`ZeroCopy`] derive.
    ///
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// Before calling `end()`, the caller must ensure that they've called
    /// [`pad::<F>()`] *in order* for every field in a struct being serialized
    /// where `F` is the type of the field. Otherwise we might not have written
    /// the necessary padding to ensure that all bytes related to the struct are
    /// initialized. Failure to do so would result in undefined behavior.
    ///
    /// Fields which are [`ZeroSized`] can be skipped.
    ///
    /// [`pad::<F>()`]: Self::pad
    /// [`ZeroSized`]: crate::traits::ZeroSized
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::buf::BufMut;
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// #[derive(Debug, PartialEq, Eq, ZeroCopy)]
    /// #[repr(C)]
    /// struct ZeroPadded(u8, u16);
    ///
    /// let padded = ZeroPadded(0x01u8.to_be(), 0x0203u16.to_be());
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let reference = Ref::<ZeroPadded>::new(buf.next_offset::<ZeroPadded>());
    ///
    /// // SAFETY: We do not pad beyond known fields and are
    /// // making sure to initialize all of the buffer.
    /// unsafe {
    ///     let mut w = buf.store_struct(&padded);
    ///     w.pad(&padded.0);
    ///     w.pad(&padded.1);
    ///     w.end();
    /// }
    ///
    /// // Note: The bytes are explicitly convert to big-endian encoding above.
    /// assert_eq!(buf.as_slice(), &[1, 0, 2, 3]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(buf.load(reference)?, &padded);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub unsafe fn end(self) {
        let count = size_of::<T>() - self.offset;
        ptr::write_bytes(self.ptr.wrapping_add(self.offset), 0, count);
    }
}
