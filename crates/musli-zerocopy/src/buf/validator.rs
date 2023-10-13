use core::marker::PhantomData;

use crate::buf::Cursor;
use crate::error::Error;
use crate::traits::ZeroCopy;

/// Validator over a [`Buf`] constructed using [`Buf::validate_struct`].
///
/// [`Buf`]: crate::buf::Buf
/// [`Buf::validate_struct`]: crate::buf::Buf::validate_struct
#[must_use = "Must call `Validator::end` when validation is completed"]
pub struct Validator<'a, T> {
    cursor: Cursor<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> Validator<'a, T> {
    #[inline]
    pub(crate) fn new(cursor: Cursor<'a>) -> Self
    where
        T: ZeroCopy,
    {
        Self {
            cursor,
            _marker: PhantomData,
        }
    }

    /// Validate an additional field in the struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned();
    ///
    /// let mut v = buf.validate_struct::<Custom>()?;
    ///
    /// // SAFETY: We're only validating fields we know are
    /// // part of the struct, and do not go beyond.
    /// unsafe {
    ///     v.field::<u32>()?;
    ///     v.field::<u64>()?;
    /// }
    ///
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn field<F>(&mut self) -> Result<&F, Error>
    where
        F: ZeroCopy,
    {
        // SAFETY: We've ensured that the provided buffer is aligned and sized
        // appropriately above.
        unsafe {
            self.cursor.align::<F>();
            F::validate(self.cursor)?;
        };

        Ok(unsafe { self.cursor.cast() })
    }
}
