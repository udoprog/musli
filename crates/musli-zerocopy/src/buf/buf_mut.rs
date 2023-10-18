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
    #[inline]
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
    #[inline]
    pub unsafe fn store_unsized_slice<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        self.start
            .copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
        let end = self.start.wrapping_add(size_of_val(values));

        if T::PADDED {
            let mut start = replace(&mut self.start, end);

            for value in values {
                let mut padder = Padder::new(start);
                T::pad(value, &mut padder);
                padder.remaining();
                start = start.wrapping_add(size_of::<T>());
            }
        } else {
            self.start = end;
        }
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
    #[inline]
    pub(crate) unsafe fn store_unaligned<T>(&mut self, value: *const T)
    where
        T: ZeroCopy,
    {
        self.start
            .copy_from_nonoverlapping(value.cast(), size_of::<T>());
        let end = self.start.wrapping_add(size_of::<T>());

        if T::PADDED {
            let start = replace(&mut self.start, end);
            let mut padder = Padder::new(start);
            T::pad(value, &mut padder);
            padder.remaining();
        } else {
            self.start = end;
        }
    }
}
