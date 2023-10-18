use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::ops::Range;
use core::ptr;

use crate::error::Error;
use crate::traits::ZeroCopy;

/// Validator over a [`Buf`] constructed using [`Buf::validate_struct`].
///
/// [`Buf`]: crate::buf::Buf
/// [`Buf::validate_struct`]: crate::buf::Buf::validate_struct
#[must_use = "Must call `Validator::end` when validation is completed"]
#[repr(transparent)]
pub struct Validator<'a, T: ?Sized> {
    data: *const u8,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Validator<'a, T> {
    #[inline]
    pub(crate) fn new(data: *const u8) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }

    /// Indicate that this validate is transparent over `U`.
    //
    /// # Safety
    ///
    /// This is only allowed if `T` is `#[repr(transparent)]` over `U`.
    #[inline]
    pub unsafe fn transparent<U>(&mut self) -> &mut Validator<'a, U> {
        transmute(self)
    }

    /// Validate an additional field in the struct and return a reference to it.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, field2: u64 }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.store(&Custom { field: 42, field2: 85 });
    /// let buf = buf.into_aligned();
    ///
    /// let mut v = buf.validate_struct::<Custom>()?;
    ///
    /// // SAFETY: We're only validating fields we know are
    /// // part of the struct, going beyond would constitute undefined behavior.
    /// unsafe {
    ///     assert_eq!(v.field::<u32>()?, &42);
    ///     assert_eq!(v.field::<u64>()?, &85);
    /// }
    ///
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// For packed structs we have to be even more careful. In fact, we're not
    /// allowed to call `field` at all and must instead solely rely on
    /// [`validate_with()`].
    ///
    /// [`validate_with()`]: Validator::validate_with
    #[inline]
    pub unsafe fn field<F>(&mut self) -> Result<&F, Error>
    where
        F: ZeroCopy,
    {
        // SAFETY: We've ensured that the provided buffer is aligned and sized
        // appropriately above.
        unsafe {
            self.align_with(align_of::<F>());
            F::validate(&mut Validator::new(self.data))?;
            let output = &*self.data.cast::<F>();
            self.advance::<F>();
            Ok(output)
        }
    }

    /// Load a single byte from the validator.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    #[inline]
    pub unsafe fn byte(&mut self) -> u8 {
        let b = ptr::read(self.data);
        self.data = self.data.wrapping_add(1);
        b
    }

    /// Perform an unaligned load of the given field.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    #[inline]
    pub unsafe fn load_unaligned<F>(&mut self) -> Result<F, Error>
    where
        F: Copy,
    {
        // SAFETY: We've ensured that the provided buffer is aligned and
        // sized
        // appropriately above.
        unsafe {
            let output = ptr::read_unaligned(self.data.cast::<F>());
            self.advance::<F>();
            Ok(output)
        }
    }

    /// Validate an additional field in the struct.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    ///
    /// # Examples
    ///
    /// Validator a packed struct:
    ///
    /// ```
    /// use std::num::NonZeroU64;
    ///
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Packed { field: u32, field2: NonZeroU64 }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// buf.store(&Packed { field: 42, field2: NonZeroU64::new(84).unwrap() });
    /// let buf = buf.into_aligned();
    ///
    /// let mut v = buf.validate_struct::<Packed>()?;
    ///
    /// // SAFETY: We're only validating fields we know are
    /// // part of the struct, and do not go beyond. We're
    /// // also making sure not to construct reference to
    /// // the fields which would be an error for a packed struct.
    /// unsafe {
    ///     v.validate::<u32>()?;
    ///     v.validate::<NonZeroU64>()?;
    /// }
    ///
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub unsafe fn validate<F>(&mut self) -> Result<(), Error>
    where
        F: ZeroCopy,
    {
        self.validate_with::<F>(align_of::<F>())
    }

    /// Validate an additional field in the struct with alignment `align`.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    ///
    /// The `align` argument must match the alignment `N` used in the
    /// `#[repr(packed(N))]` argument, note that `#[repr(packed)]` has an
    /// argument of 1.
    ///
    /// # Examples
    ///
    /// Validator a packed struct:
    ///
    /// ```
    /// use std::num::NonZeroU64;
    ///
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C, packed(2))]
    /// struct Packed { field: u32, field2: NonZeroU64 }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// buf.store(&Packed { field: 42, field2: NonZeroU64::new(84).unwrap() });
    /// let buf = buf.into_aligned();
    ///
    /// let mut v = buf.validate_struct::<Packed>()?;
    ///
    /// // SAFETY: We're only validating fields we know are
    /// // part of the struct, and do not go beyond. We're
    /// // also making sure not to construct reference to
    /// // the fields which would be an error for a packed struct.
    /// unsafe {
    ///     v.validate_with::<u32>(2)?;
    ///     v.validate_with::<NonZeroU64>(2)?;
    /// }
    ///
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub unsafe fn validate_with<F>(&mut self, align: usize) -> Result<(), Error>
    where
        F: ZeroCopy,
    {
        self.align_with(align);
        F::validate(&mut Validator::new(self.data))?;
        self.advance::<F>();
        Ok(())
    }

    /// Only validate the given field without aligning it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the field is properly
    /// aligned already by for example calling [`align::<F>()`].
    ///
    /// [`align::<F>()`]: Self::align
    #[inline]
    pub(crate) unsafe fn validate_only<F>(&mut self) -> Result<(), Error>
    where
        F: ZeroCopy,
    {
        // SAFETY: We've ensured that the provided buffer is aligned and sized
        // appropriately above.
        F::validate(&mut Validator::new(self.data))?;
        self.advance::<F>();
        Ok(())
    }

    /// Align the current pointer by `F`.
    #[inline]
    pub(crate) unsafe fn align_with(&mut self, align: usize) {
        let offset = self.data.align_offset(align);
        self.data = self.data.wrapping_add(offset);
    }

    /// Advance the current pointer by `F`.
    #[inline]
    pub(crate) unsafe fn advance<F>(&mut self) {
        self.data = self.data.wrapping_add(size_of::<F>());
    }

    /// Return the address range associated with a just read `F` for diagnostics.
    #[inline]
    pub(crate) fn range<F>(&self) -> Range<usize> {
        let end = self.data as usize;
        let start = end.wrapping_sub(size_of::<F>());
        start..end
    }
}
