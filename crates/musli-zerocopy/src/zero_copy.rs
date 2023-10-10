#![allow(clippy::len_without_is_empty)]

use core::marker::PhantomData;
use core::mem::align_of;
use core::str;

use crate::buf::{AnyValue, Buf, BufMut};
use crate::error::{Error, ErrorKind};

/// Trait governing which `T` in [`Unsized<T>`] the wrapper can handle.
///
/// We only support slice-like, unaligned unsized types, such as `str` and
/// `[u8]`. We can't support types such as `dyn Debug` because metadata is a
/// vtable which can't be serialized.
///
/// For nested slices or arrays, use [`Slice<T>`] instead.
///
/// [`Unsized<T>`]: crate::unsized::Unsized
/// [`Slice<T>`]: crate::slice::Slice
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
/// * The base type has a statically known alignment, such as how `[u8]` is
///   aligned on 1 bytes and this alignment must be specified in `ALIGN`.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
///
/// let mut buf = AlignedBuf::with_alignment(1);
///
/// let bytes = buf.write_unsized(&b"Hello World!"[..])?;
/// let buf = buf.as_ref()?;
/// assert_eq!(buf.load(bytes)?, b"Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub unsafe trait UnsizedZeroCopy {
    /// Alignment of the pointed to data. We can only support unsized types
    /// which have a known alignment.
    const ALIGN: usize;

    /// The size in bytes of the pointed to value.
    fn size(&self) -> usize;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Validate and coerce the buffer as this type.
    fn coerce(buf: &Buf) -> Result<&Self, Error>;

    /// Validate and coerce the buffer as this type mutably.
    fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error>;
}

/// This is a marker trait that must be implemented for a type in order to use
/// the [`#[zero_copy(ignore)]`] attribute when deriving the [`ZeroCopy`] trait.
///
/// Using the attribute incorrectly might lead to unsoundness.
pub unsafe trait ZeroSized {}

// SAFETY: `()` is zero-sized.
unsafe impl ZeroSized for () {}

// SAFETY: `PhantomData<T>` is zero-sized.
unsafe impl<T> ZeroSized for PhantomData<T> {}

/// Trait governing how to write a sized buffer.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait ZeroCopy {
    /// Indicates if the type can inhabit all possible bit patterns within its
    /// `size_of::<Self>()` bytes.
    const ANY_BITS: bool;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Coerce and validate the buffer as reference of this type.
    fn coerce(buf: &Buf) -> Result<&Self, Error>;

    /// Coerce and validate the buffer as a mutable reference of this type.
    fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error>;

    /// Only validate the provided buffer.
    ///
    /// # Safety
    ///
    /// This assumes that the provided buffer is correctly sized and aligned,
    /// something the caller is responsible for ensuring.
    ///
    /// ```no_run
    /// use core::mem::align_of;
    ///
    /// use musli_zerocopy::{AlignedBuf, Buf, Error, Ptr, Ref, ZeroCopy};
    ///
    /// unsafe fn unsafe_coerce<T>(buf: &Buf) -> Result<&T, Error>
    /// where
    ///     T: ZeroCopy
    /// {
    ///     // SAFETY: We've checked that the buffer is compatible.
    ///     T::validate(buf)?;
    ///     Ok(buf.cast())
    /// }
    ///
    /// let mut buf = AlignedBuf::with_alignment(align_of::<u32>());
    /// buf.write(&42u32)?;
    ///
    /// let buf = buf.as_ref()?;
    ///
    /// // Safe variant which performs layout checks for us.
    /// assert_eq!(buf.load(Ref::<u32>::zero())?, &42);
    ///
    /// // Achieves the same as above, but we take on the responsibility
    /// // of never using it with improperly aligned or sized buffers.
    /// unsafe {
    ///     assert_eq!(unsafe_coerce::<u32>(buf)?, &42);
    /// }
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    unsafe fn validate(buf: &Buf) -> Result<(), Error>;
}

unsafe impl UnsizedZeroCopy for str {
    const ALIGN: usize = align_of::<u8>();

    #[inline]
    fn size(&self) -> usize {
        <str>::len(self)
    }

    #[inline]
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self.as_bytes())
    }

    #[inline]
    fn coerce(buf: &Buf) -> Result<&Self, Error> {
        str::from_utf8(buf.as_slice()).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))
    }

    #[inline]
    fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
        str::from_utf8_mut(buf.as_mut_slice())
            .map_err(|error| Error::new(ErrorKind::Utf8Error { error }))
    }
}

unsafe impl UnsizedZeroCopy for [u8] {
    const ALIGN: usize = align_of::<u8>();

    #[inline]
    fn size(&self) -> usize {
        <[_]>::len(self)
    }

    #[inline]
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self)
    }

    #[inline]
    fn coerce(buf: &Buf) -> Result<&Self, Error> {
        Ok(buf.as_slice())
    }

    #[inline]
    fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
        Ok(buf.as_mut_slice())
    }
}

macro_rules! impl_number {
    ($ty:ty) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        /// use std::slice;
        /// use std::mem::size_of;
        /// use musli_zerocopy::{ZeroCopy, Buf, Ptr, Ref};
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($ty), ",")]
        /// }
        ///
        #[doc = concat!("let size = size_of::<", stringify!($ty) ,">();")]
        ///
        #[doc = concat!("let zero: ", stringify!($ty), " = 0;")]
        #[doc = concat!("let one: ", stringify!($ty), " = 1;")]
        ///
        /// let zero = unsafe {
        ///     let bytes = slice::from_raw_parts(&zero as *const _ as *const u8, size);
        ///     Buf::new(bytes)
        /// };
        ///
        /// let one = unsafe {
        ///     let bytes = slice::from_raw_parts(&one as *const _ as *const u8, size);
        ///     Buf::new(bytes)
        /// };
        ///
        /// assert_eq!(zero.load(Ref::<Struct>::new(Ptr::ZERO))?.field, 0);
        /// assert_eq!(one.load(Ref::<Struct>::new(Ptr::ZERO))?.field, 1);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;

            fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                buf.extend_from_slice(&<$ty>::to_ne_bytes(*self)[..])
            }

            fn coerce(buf: &Buf) -> Result<&Self, Error> {
                buf.ensure_compatible_with::<Self>()?;
                Ok(unsafe { buf.cast() })
            }

            fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
                buf.ensure_compatible_with::<Self>()?;
                Ok(unsafe { buf.cast_mut() })
            }

            // NB: Numerical types can inhabit any bit pattern.
            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate(_: &Buf) -> Result<(), Error> {
                Ok(())
            }
        }

        impl AnyValue for $ty {
            type Target = $ty;

            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
            }
        }
    };
}

impl_number!(usize);
impl_number!(isize);
impl_number!(u8);
impl_number!(u16);
impl_number!(u32);
impl_number!(u64);
impl_number!(u128);
impl_number!(i8);
impl_number!(i16);
impl_number!(i32);
impl_number!(i64);
impl_number!(i128);

macro_rules! impl_nonzero_number {
    ($ty:ident, $example:ty) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("use std::num::", stringify!($ty), ";")]
        /// use std::slice;
        /// use std::mem::size_of;
        /// use musli_zerocopy::{ZeroCopy, Buf, Ptr, Ref};
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($ty), ",")]
        /// }
        ///
        #[doc = concat!("let size = size_of::<", stringify!($example) ,">();")]
        ///
        #[doc = concat!("let zero: ", stringify!($example), " = 0;")]
        #[doc = concat!("let one: ", stringify!($example), " = 1;")]
        ///
        /// let zero = unsafe {
        ///     let bytes = slice::from_raw_parts(&zero as *const _ as *const u8, size);
        ///     Buf::new(bytes)
        /// };
        ///
        /// let one = unsafe {
        ///     let bytes = slice::from_raw_parts(&one as *const _ as *const u8, size);
        ///     Buf::new(bytes)
        /// };
        ///
        /// // Non-zero buffer works as expected.
        /// assert_eq!(one.load(Ref::<Struct>::new(Ptr::ZERO))?.field.get(), 1);
        ///
        /// // Trying to use a zeroed buffer with a non-zero type.
        /// assert!(zero.load(Ref::<Struct>::new(Ptr::ZERO)).is_err());
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for ::core::num::$ty {
            const ANY_BITS: bool = false;

            fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                buf.extend_from_slice(&self.get().to_ne_bytes()[..])
            }

            fn coerce(buf: &Buf) -> Result<&Self, Error> {
                buf.ensure_compatible_with::<::core::num::$ty>()?;

                // SAFETY: Layout has been checked.
                unsafe {
                    Self::validate(buf)?;
                }

                Ok(unsafe { buf.cast() })
            }

            fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
                buf.ensure_compatible_with::<::core::num::$ty>()?;

                // SAFETY: Layout has been checked.
                unsafe {
                    Self::validate(buf)?;
                }

                Ok(unsafe { buf.cast_mut() })
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate(buf: &Buf) -> Result<(), Error> {
                if buf.is_zeroed() {
                    return Err(Error::new(ErrorKind::NonZeroZeroed { range: buf.range() }));
                }

                Ok(())
            }
        }

        impl AnyValue for ::core::num::$ty {
            type Target = ::core::num::$ty;

            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
            }
        }
    };
}

impl_nonzero_number!(NonZeroUsize, usize);
impl_nonzero_number!(NonZeroIsize, isize);
impl_nonzero_number!(NonZeroU8, u8);
impl_nonzero_number!(NonZeroU16, u16);
impl_nonzero_number!(NonZeroU32, u32);
impl_nonzero_number!(NonZeroU64, u64);
impl_nonzero_number!(NonZeroU128, u128);
impl_nonzero_number!(NonZeroI8, i8);
impl_nonzero_number!(NonZeroI16, i16);
impl_nonzero_number!(NonZeroI32, i32);
impl_nonzero_number!(NonZeroI64, i64);
impl_nonzero_number!(NonZeroI128, i128);

macro_rules! impl_zst {
    ($({$name:ident},)? $ty:ty, $expr:expr , {$example:ty $(, $import:path)?}) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        $(#[doc = concat!("use ", stringify!($import), ";")])*
        /// use musli_zerocopy::{ZeroCopy, Slice, AlignedBuf};
        ///
        /// #[derive(Default, Clone, Copy, ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($example), ",")]
        /// }
        ///
        /// let mut empty = AlignedBuf::new();
        /// let values = [Struct::default(); 100];
        /// let slice = empty.write_slice(&values[..])?;
        /// let buf = empty.as_aligned();
        /// assert_eq!(buf.len(), 0);
        ///
        /// let slice = buf.load(slice)?;
        /// assert_eq!(slice.len(), 100);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl $(<$name>)* ZeroCopy for $ty {
            const ANY_BITS: bool = true;

            #[inline]
            fn write_to<B: ?Sized>(&self, _: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                Ok(())
            }

            #[inline]
            fn coerce(buf: &Buf) -> Result<&Self, Error> {
                Ok(unsafe { buf.cast() })
            }

            #[inline]
            fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
                Ok(unsafe { buf.cast_mut() })
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate(_: &Buf) -> Result<(), Error> {
                Ok(())
            }
        }

        impl $(<$name>)* AnyValue for $ty {
            type Target = $ty;

            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
            }
        }
    };
}

impl_zst!((), (), { () });
impl_zst!({T}, PhantomData<T>, PhantomData, {PhantomData<u32>, std::marker::PhantomData});

unsafe impl<T, const N: usize> ZeroCopy for [T; N]
where
    T: ZeroCopy,
{
    const ANY_BITS: bool = T::ANY_BITS;

    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        for element in self {
            element.write_to(buf)?;
        }

        Ok(())
    }

    fn coerce_mut(buf: &mut Buf) -> Result<&mut Self, Error> {
        crate::buf::validate_array::<T>(buf, N)?;
        // SAFETY: All preconditions above have been tested.
        Ok(unsafe { buf.cast_mut() })
    }

    fn coerce(buf: &Buf) -> Result<&Self, Error> {
        crate::buf::validate_array::<T>(buf, N)?;
        // SAFETY: All preconditions above have been tested.
        Ok(unsafe { buf.cast() })
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn validate(buf: &Buf) -> Result<(), Error> {
        crate::buf::validate_array_aligned::<T>(buf)?;
        Ok(())
    }
}
