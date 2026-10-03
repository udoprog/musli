use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::ptr::{self, NonNull};

use crate::buf::padding_to;
use crate::error::{Error, ErrorKind};
use crate::traits::{ZeroCopy, ZeroSized};

/// Validator over a [`Buf`] constructed using [`Buf::validate_struct`].
///
/// Fields are aligned relative to the start of the value being validated,
/// which matches how the compiler lays out fields. The value itself is not
/// required to be aligned, since it might for example be stored inside of a
/// `#[repr(packed)]` struct or be loaded through an unaligned load.
///
/// [`Buf`]: crate::buf::Buf
/// [`Buf::validate_struct`]: crate::buf::Buf::validate_struct
#[must_use = "Must call `Validator::end` when validation is completed"]
// NB: `repr(C)` ensures that `Validator<T>` and `Validator<U>` have the same
// layout, which `transparent` relies on.
#[repr(C)]
pub struct Validator<'a, T: ?Sized> {
    /// The start of the value being validated.
    data: NonNull<u8>,
    /// The offset of the next field relative to `data`.
    offset: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Validator<'a, T> {
    /// Construct a validator around a slice.
    ///
    /// This does not require that the slice is a valid instance of `T`.
    pub(crate) fn from_slice(slice: &[u8]) -> Self {
        // SAFETY: a slice is guaranteed to be non-null.
        unsafe { Self::new(NonNull::new_unchecked(slice.as_ptr() as *mut u8)) }
    }

    /// Construct a validator around a pointer.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the pointer points to an initialized slice of
    /// size `T`.
    #[inline]
    pub(crate) unsafe fn new(data: NonNull<u8>) -> Self {
        Self {
            data,
            offset: 0,
            _marker: PhantomData,
        }
    }

    /// Pointer to the current field.
    #[inline]
    fn current(&self) -> NonNull<u8> {
        // SAFETY: The caller of the methods which advance the offset ensure
        // that it stays within the value being validated.
        unsafe { self.data.add(self.offset) }
    }

    /// Indicate that this validate is transparent over `U`.
    //
    /// # Safety
    ///
    /// This is only allowed if `T` is `#[repr(transparent)]` over `U`.
    #[inline]
    pub unsafe fn transparent<U>(&mut self) -> &mut Validator<'a, U> {
        // SAFETY: `Validator` is `repr(C)` and its layout does not depend on
        // `T`.
        unsafe { &mut *(self as *mut Self).cast::<Validator<'a, U>>() }
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
    /// let custom = buf.store(&Custom { field: 42, field2: 85 })?;
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
    /// # Errors
    ///
    /// Since fields are aligned relative to the start of the value being
    /// validated, this errors if the field is not aligned in memory. This can
    /// happen if the value is not itself aligned, such as when it is stored
    /// inside of a packed struct.
    ///
    /// [`validate_with()`]: Validator::validate_with
    #[inline]
    pub unsafe fn field<F>(&mut self) -> Result<&F, Error>
    where
        F: ZeroCopy,
    {
        unsafe {
            self.align_with(align_of::<F>());
            let ptr = self.current();

            if !ptr.cast::<F>().is_aligned() {
                let addr = ptr.as_ptr() as usize;

                return Err(Error::new(ErrorKind::AlignmentRangeMismatch {
                    addr,
                    range: addr..addr.wrapping_add(size_of::<F>()),
                    align: align_of::<F>(),
                }));
            }

            F::validate(&mut Validator::new(ptr))?;
            // SAFETY: We've checked that the pointer is aligned above, and
            // the caller ensures that it is in bounds.
            let output = ptr.cast::<F>().as_ref();
            self.advance::<F>();
            Ok(output)
        }
    }

    /// Align, validate and perform an unaligned load of an additional field.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    #[inline]
    pub(crate) unsafe fn read_field<F>(&mut self) -> Result<F, Error>
    where
        F: ZeroCopy + Copy,
    {
        unsafe {
            self.align_with(align_of::<F>());
            let ptr = self.current();
            F::validate(&mut Validator::new(ptr))?;
            let output = ptr::read_unaligned(ptr.cast::<F>().as_ptr());
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
        unsafe {
            let b = ptr::read(self.current().as_ptr());
            self.offset += 1;
            b
        }
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
        // SAFETY: The caller ensures that the field is in bounds.
        unsafe {
            let output = ptr::read_unaligned(self.current().cast::<F>().as_ptr());
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
    /// buf.store(&Packed { field: 42, field2: NonZeroU64::new(84).unwrap() })?;
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
        unsafe { self.validate_with::<F>(align_of::<F>()) }
    }

    /// Validate an additional field in a struct marked
    /// `#[repr(packed(align))]`.
    ///
    /// The field is aligned to `min(align, align_of::<F>())`, which is the
    /// alignment the compiler uses for fields in a packed struct.
    ///
    /// # Safety
    ///
    /// The current validator only guarantees that validation up to the size of
    /// `T` can be performed. Advancing beyond that size causes the validator to
    /// walk out of bounds.
    ///
    /// The `align` argument must match the alignment `N` used in the
    /// `#[repr(packed(N))]` argument, note that `#[repr(packed)]` has an
    /// argument of 1. `align` must be a power of two.
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
    /// buf.store(&Packed { field: 42, field2: NonZeroU64::new(84).unwrap() })?;
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
        unsafe {
            self.align_with(align.min(align_of::<F>()));
            F::validate(&mut Validator::new(self.current()))?;
            self.advance::<F>();
            Ok(())
        }
    }

    /// Skip over an ignored zero-sized field `F`.
    ///
    /// Zero-sized fields can have an alignment larger than 1, in which case
    /// they affect the offset of the fields that follow them.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being validated.
    #[inline]
    pub unsafe fn validate_zero_sized<F>(&mut self)
    where
        F: ZeroSized,
    {
        unsafe {
            self.align_with(align_of::<F>());
        }
    }

    /// Skip over an ignored zero-sized field `F` in a struct marked
    /// `#[repr(packed(align))]`.
    ///
    /// The field is aligned to `min(align, align_of::<F>())`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field type `F` is an actual field in
    /// order in the struct being validated and that `align` matches the
    /// argument provided to `#[repr(packed)]` (note that empty means 1).
    /// `align` must be a power of two.
    #[inline]
    pub unsafe fn validate_zero_sized_with<F>(&mut self, align: usize)
    where
        F: ZeroSized,
    {
        unsafe {
            self.align_with(align.min(align_of::<F>()));
        }
    }

    /// Only validate the given field without aligning it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the field is properly
    /// aligned already by for example calling [`align_with::<F>()`].
    ///
    /// [`align_with::<F>()`]: Self::align_with
    #[inline]
    pub(crate) unsafe fn validate_only<F>(&mut self) -> Result<(), Error>
    where
        F: ZeroCopy,
    {
        unsafe {
            F::validate(&mut Validator::new(self.current()))?;
            self.advance::<F>();
            Ok(())
        }
    }

    /// Align the current offset to `align` relative to the start of the value
    /// being validated.
    ///
    /// `align` must be a power of two.
    #[inline]
    pub(crate) unsafe fn align_with(&mut self, align: usize) {
        self.offset += padding_to(self.offset, align);
    }

    /// Advance the current offset by the size of `F`.
    #[inline]
    pub(crate) unsafe fn advance<F>(&mut self) {
        self.offset += size_of::<F>();
    }

    /// Return the address range associated with a just read `F` for diagnostics.
    #[inline]
    pub(crate) fn range<F>(&self) -> Range<usize> {
        let end = (self.data.as_ptr() as usize).wrapping_add(self.offset);
        let start = end.wrapping_sub(size_of::<F>());
        start..end
    }
}
