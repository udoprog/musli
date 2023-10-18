use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::ptr;

use crate::traits::ZeroCopy;

/// A struct padder as provided to the [`ZeroCopy::pad`] method.
///
/// This knows how to find and initialize padding regions in `repr(C)` types,
/// and provides a builder-like API to doing so.
#[must_use = "For the writer to have an effect on `OwnedBuf` you must call `Padder::finish`"]
pub struct Padder<'a, T> {
    ptr: *mut u8,
    offset: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Padder<'a, T>
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

    /// Indicate that this validate is transparent over `U`.
    //
    /// # Safety
    ///
    /// This is only allowed if `T` is `#[repr(transparent)]` over `U`.
    #[inline]
    pub unsafe fn transparent<U>(&mut self) -> &mut Padder<'a, U> {
        transmute(self)
    }

    /// Pad around the given field with zeros.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// safety requirements by [`remaining()`].
    ///
    /// This is typically not called directly, but rather is implemented by the
    /// [`ZeroCopy`] derive.
    ///
    /// [`remaining()`]: Self::remaining
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being padded.
    #[inline]
    pub unsafe fn pad<F>(&mut self, field: *const F)
    where
        F: ZeroCopy,
    {
        self.pad_with(field, align_of::<F>());
    }

    /// Pad around the given field with zeros using a custom alignment `align`.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// safety requirements by [`remaining()`].
    ///
    /// This is typically not called directly, but rather is implemented by the
    /// [`ZeroCopy`] derive.
    ///
    /// [`remaining()`]: Self::remaining
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being padded and that `align` matches the argument
    /// provided to `#[repr(packed)]` (note that empty means 1).
    #[inline]
    pub unsafe fn pad_with<F>(&mut self, field: *const F, align: usize)
    where
        F: ZeroCopy,
    {
        let count = crate::buf::padding_to(self.offset, align);
        // zero out padding.
        ptr::write_bytes(self.ptr.add(self.offset), 0, count);
        self.offset = self.offset.wrapping_add(count);

        if F::PADDED {
            let mut padder = Padder::new(self.ptr.wrapping_add(self.offset));
            F::pad(field, &mut padder);
            padder.remaining();
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
    /// Before calling `remaining()`, the caller must ensure that they've called
    /// [`pad::<F>()`] *in order* for every field in a struct being serialized
    /// where `F` is the type of the field. Otherwise we might not have written
    /// the necessary padding to ensure that all bytes related to the struct are
    /// initialized. Failure to do so would result in undefined behavior.
    ///
    /// Fields which are [`ZeroSized`] can be skipped.
    ///
    /// [`pad::<F>()`]: Self::pad
    /// [`ZeroSized`]: crate::traits::ZeroSized
    #[inline]
    pub unsafe fn remaining(self) {
        let count = size_of::<T>() - self.offset;
        ptr::write_bytes(self.ptr.wrapping_add(self.offset), 0, count);
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use anyhow::Result;

    use crate::{OwnedBuf, ZeroCopy};

    #[test]
    fn ensure_padding() -> Result<()> {
        #[derive(Debug, PartialEq, Eq, ZeroCopy)]
        #[repr(C)]
        #[zero_copy(crate)]
        struct ZeroPadded(u8, u16, u64);

        let padded = ZeroPadded(
            0x01u8.to_be(),
            0x0203u16.to_be(),
            0x0405060708090a0bu64.to_be(),
        );

        let mut buf = OwnedBuf::new();

        // Note: You're responsible for ensuring that the buffer has enough
        // capacity.
        buf.reserve(size_of::<ZeroPadded>());

        // SAFETY: We do not pad beyond known fields and are making sure to
        // initialize all of the buffer.
        unsafe {
            buf.as_buf_mut().store_unaligned(&padded);
            buf.advance(size_of::<ZeroPadded>());
        }

        // Note: The bytes are explicitly convert to big-endian encoding above.
        assert_eq!(
            buf.as_slice(),
            &[1, 0, 2, 3, 0, 0, 0, 0, 4, 5, 6, 7, 8, 9, 10, 11]
        );

        Ok(())
    }
}
