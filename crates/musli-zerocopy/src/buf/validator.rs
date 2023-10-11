use core::marker::PhantomData;
use core::mem::{align_of, size_of};

use crate::buf::Buf;
use crate::error::{Error, ErrorKind};
use crate::traits::ZeroCopy;

/// Validator over a [`Buf`] constructed using [`Buf::validate`].
#[must_use = "Must call `Validator::end` when validation is completed"]
pub struct Validator<'a, T> {
    data: &'a Buf,
    offset: usize,
    _marker: PhantomData<T>,
}

impl<'a, T> Validator<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a Buf) -> Self
    where
        T: ZeroCopy,
    {
        Self {
            data,
            offset: 0,
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
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// v.end()?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn field<F>(&mut self) -> Result<&F, Error>
    where
        F: ZeroCopy,
    {
        let start = self.offset.next_multiple_of(align_of::<F>());
        let end = start.wrapping_add(size_of::<F>());
        let data = self.data.get_unaligned(start, end)?;

        // SAFETY: We've ensured that the provided buffer is aligned and sized
        // appropriately above.
        unsafe {
            F::validate(data)?;
        };

        self.offset = end;
        Ok(unsafe { data.cast() })
    }

    /// Finish validation.
    ///
    /// # Errors
    ///
    /// If there is any buffer left uncomsumed at this point, this will return
    /// an [`Error`].
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
    /// buf.extend_from_slice(&[0]);
    /// let buf = buf.as_aligned();
    ///
    /// // We can only cause the error if we assert that the buffer is aligned.
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// // Will error since the buffer is too large.
    /// assert!(v.end().is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
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
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// v.end()?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn end(self) -> Result<(), Error> {
        let offset = self.offset.next_multiple_of(size_of::<T>());

        if offset != self.data.len() {
            return Err(Error::new(ErrorKind::BufferUnderflow {
                range: self.data.range(),
                expected: offset,
            }));
        }

        Ok(())
    }
}
