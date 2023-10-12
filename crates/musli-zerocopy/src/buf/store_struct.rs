use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;

use crate::error::Error;
use crate::traits::ZeroCopy;

/// A writer as returned from [`BufMut::store_struct`].
///
/// [`BufMut::store_struct`]: crate::buf::BufMut::store_struct
#[must_use = "For the writer to have an effect on `AlignedBuf` you must call `StoreStruct::finish`"]
pub struct StoreStruct<'a, T> {
    start: *mut u8,
    end: *mut u8,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StoreStruct<'a, T>
where
    T: ZeroCopy,
{
    #[inline]
    pub(crate) fn new(start: *mut u8, end: *mut u8) -> Self {
        Self {
            start,
            end,
            _marker: PhantomData,
        }
    }

    /// Pad around the given field with zeros.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// safety requirements by [`finish()`].
    ///
    /// This is typically not called directly, but rather is implemented by the
    /// [`ZeroCopy`] derive.
    ///
    /// [`finish()`]: Self::finish
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::buf::StoreStruct;
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
    ///     w.pad::<u8>();
    ///     w.pad::<u16>();
    ///     w.finish();
    /// }
    ///
    /// // Note: The bytes are explicitly convert to big-endian encoding above.
    /// assert_eq!(buf.as_slice(), &[1, 0, 2, 3]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn pad<F>(&mut self)
    where
        F: ZeroCopy,
    {
        unsafe {
            let offset = self.start.align_offset(align_of::<F>());

            // zero out padding.
            if offset > 0 {
                ptr::write_bytes(self.start, 0, offset);
            }

            self.start = self.start.wrapping_add(offset).wrapping_add(size_of::<F>());
        }
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
    /// Before calling `finish()`, the caller must ensure that they've called
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
    /// use musli_zerocopy::buf::StoreStruct;
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
    ///     w.pad::<u8>();
    ///     w.pad::<u16>();
    ///     w.finish();
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
    pub unsafe fn finish(self) -> Result<(), Error> {
        let distance = self.start.offset_from(self.end);

        if distance > 0 {
            ptr::write_bytes(self.start, 0, distance as usize);
        }

        Ok(())
    }
}
