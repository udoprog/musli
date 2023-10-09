#![allow(clippy::len_without_is_empty)]

use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::str;

use crate::buf::{AnyValue, Buf, BufMut};
use crate::error::{Error, ErrorKind};

/// Trait governing how to write an unsized buffer.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
/// * The base type has a statically known alignment, such as how `[u32]` is
///   aligned on 4 bytes.
pub unsafe trait UnsizedZeroCopy {
    /// Alignment of the pointed to data. We can only support unsized types
    /// which have a known alignment.
    const ALIGN: usize;

    /// The length of the unsized value.
    fn len(&self) -> usize;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Validate the buffer as this type.
    fn read_from(buf: &Buf) -> Result<&Self, Error>;
}

/// Trait governing how to write a sized buffer.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait ZeroCopy: Sized {
    /// Size of the pointed to data.
    const SIZE: usize = mem::size_of::<Self>();

    /// Alignment of the pointed to data.
    const ALIGN: usize = mem::align_of::<Self>();

    /// Indicates if the type can inhabit all possible bit patterns within its
    /// `SIZE`.
    ///
    /// By default ZSTs set this as true.
    const ANY_BITS: bool = mem::size_of::<Self>() == 0;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Validate the buffer as this type.
    fn read_from(buf: &Buf) -> Result<&Self, Error>;

    /// Just validate the current buffer under the assumption that it is
    /// correctly aligned for the current type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized and aligned
    /// per the requirements of this type.
    unsafe fn validate_aligned(buf: &Buf) -> Result<(), Error>;
}

unsafe impl UnsizedZeroCopy for str {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <str>::len(self)
    }

    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self.as_bytes())
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        str::from_utf8(buf.as_bytes()).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))
    }
}

unsafe impl UnsizedZeroCopy for [u8] {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self)
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        Ok(buf.as_bytes())
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
        ///     Buf::new_unchecked(bytes)
        /// };
        ///
        /// let one = unsafe {
        ///     let bytes = slice::from_raw_parts(&one as *const _ as *const u8, size);
        ///     Buf::new_unchecked(bytes)
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

            fn read_from(buf: &Buf) -> Result<&Self, Error> {
                if !buf.is_compatible(core::alloc::Layout::new::<$ty>()) {
                    return Err(Error::new(ErrorKind::LayoutMismatch {
                        layout: Layout::new::<$ty>(),
                        buf: buf.range(),
                    }));
                }

                Ok(unsafe { buf.cast() })
            }

            // NB: Numerical types can inhabit any bit pattern.
            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate_aligned(_: &Buf) -> Result<(), Error> {
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
        ///     Buf::new_unchecked(bytes)
        /// };
        ///
        /// let one = unsafe {
        ///     let bytes = slice::from_raw_parts(&one as *const _ as *const u8, size);
        ///     Buf::new_unchecked(bytes)
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
            fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                buf.extend_from_slice(&self.get().to_ne_bytes()[..])
            }

            fn read_from(buf: &Buf) -> Result<&Self, Error> {
                if !buf.is_compatible(core::alloc::Layout::new::<::core::num::$ty>()) {
                    return Err(Error::new(ErrorKind::LayoutMismatch {
                        layout: Layout::new::<::core::num::$ty>(),
                        buf: buf.range(),
                    }));
                }

                if buf.is_zeroed() {
                    return Err(Error::new(ErrorKind::NonZeroZeroed { range: buf.range() }));
                }

                Ok(unsafe { buf.cast() })
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate_aligned(buf: &Buf) -> Result<(), Error> {
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
        /// use musli_zerocopy::{ZeroCopy, SliceRef, OwnedBuf};
        ///
        /// #[derive(Default, Clone, Copy, ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($example), ",")]
        /// }
        ///
        /// let mut empty = OwnedBuf::new();
        /// let values = [Struct::default(); 100];
        /// let slice = empty.insert_slice(&values[..])?;
        /// let buf = empty.as_aligned_buf();
        /// assert_eq!(buf.len(), 0);
        ///
        /// let slice = buf.load(slice)?;
        /// assert_eq!(slice.len(), 100);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl $(<$name>)* ZeroCopy for $ty {
            fn write_to<B: ?Sized>(&self, _: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                Ok(())
            }

            fn read_from(_: &Buf) -> Result<&Self, Error> {
                Ok(&$expr)
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn validate_aligned(_: &Buf) -> Result<(), Error> {
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
