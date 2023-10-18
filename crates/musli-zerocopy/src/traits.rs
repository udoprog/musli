//! Traits that apply to types which can safely interact with Müsli's zero copy
//! system.
//!
//! Note that all of these traits are `unsafe`, and require care to implement.
//! Please see their corresponding safety documentation or use the
//! [`ZeroCopy`][derive@crate::ZeroCopy] derive.
//!
//! * [`ZeroCopy`] for types which can safely be coerced from a [`Buf`] to
//!   `&Self` or `&mut Self`.
//! * [`UnsizedZeroCopy`] for types which can safely be coerced from an
//!   [`Unsized<T>`] to `&T` or `&mut T`.
//! * [`ZeroSized`] for types which can be ingored when deriving
//!   [`ZeroCopy`][derive@crate::ZeroCopy] using `#[zero_copy(ignore)]`.
//!
//! [`Unsized<T>`]: crate::pointer::Unsized

#![allow(clippy::missing_safety_doc)]

use core::marker::PhantomData;
use core::mem::{align_of, size_of, size_of_val};
use core::num::Wrapping;
use core::slice;
use core::str;

use crate::buf::{Buf, BufMut, Padder, Validator, Visit};
use crate::error::{Error, ErrorKind};

mod sealed {
    pub trait Sealed {}
    impl Sealed for str {}
}

/// Trait governing which `T` in [`Unsized<T>`] the wrapper can handle.
///
/// We only support slice-like, unaligned unsized types, such as `str` and
/// `[u8]`. We can't support types such as `dyn Debug` because metadata is a
/// vtable which can't be serialized.
///
/// For nested slices or arrays, use [`Slice<T>`] instead.
///
/// [`Unsized<T>`]: crate::pointer::Unsized
/// [`Slice<T>`]: crate::pointer::Slice
///
/// # Safety
///
/// This can only be implemented by types that:
/// * Can only be implemented for base types which can inhabit any bit-pattern.
///   All though custom validation can be performed during coercion (such as for
///   `str`).
/// * Must only be implemented for types which are not padded (as per
///   [`ZeroCopy::PADDED`]).
///
/// # Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
///
/// let mut buf = OwnedBuf::with_alignment::<u8>();
///
/// let bytes = buf.store_unsized(&b"Hello World!"[..]);
/// let buf = buf.as_ref();
/// assert_eq!(buf.load(bytes)?, b"Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub unsafe trait UnsizedZeroCopy: self::sealed::Sealed {
    /// Alignment of the pointed to data. We can only support unsized types
    /// which have a known alignment.
    ///
    /// # Safety
    ///
    /// This must be a power of two.
    const ALIGN: usize;

    /// The size of the base type.
    const SIZE: usize;

    /// The size of the unsized value.
    fn size(&self) -> usize;

    /// The size in bytes of the unsized value.
    fn bytes(&self) -> usize;

    /// Write to the owned buffer.
    ///
    /// This is usually called indirectly through methods such as
    /// [`OwnedBuf::store_unsized`].
    ///
    /// [`OwnedBuf::store_unsized`]: crate::buf::OwnedBuf::store_unsized
    unsafe fn store(&self, buf: &mut BufMut<'_>);

    /// Validate and coerce the buffer as this type.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the pointer is valid up to
    /// the reported size of `Self`. If `Self` is `[T]` then `size` is the
    /// length of the `T`-containing slice.
    unsafe fn coerce(buf: *const u8, size: usize) -> Result<*const Self, Error>;

    /// Validate and coerce the buffer as this type mutably.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the pointer is valid up to
    /// the reported size of `Self`. If `Self` is `[T]` then `size` is the
    /// length of the `T`-containing slice.
    unsafe fn coerce_mut(buf: *mut u8, size: usize) -> Result<*mut Self, Error>;
}

/// This is a marker trait that must be implemented for a type in order to use
/// the `#[zero_copy(ignore)]` attribute when deriving the [`ZeroCopy`] trait.
///
/// Using the attribute incorrectly might lead to unsoundness.
///
/// # Safety
///
/// Any type implementing this trait must be zero-sized.
///
/// # Examples
///
/// Using `#[zero_copy(ignore)]`` on generic fields that implements
/// [`ZeroSized`]:
///
/// ```
/// use musli_zerocopy::ZeroCopy;
/// use musli_zerocopy::traits::ZeroSized;
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct Struct<T> where T: ZeroSized {
///     #[zero_copy(ignore)]
///     field: T,
/// }
/// ```
///
/// Types which derive [`ZeroCopy`] also implement [`ZeroSized`] if they are
/// zero-sized:
///
/// ```
/// use std::marker::PhantomData;
/// use std::mem::size_of;
/// use musli_zerocopy::ZeroCopy;
/// use musli_zerocopy::traits::ZeroSized;
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct Struct<T> where T: ZeroSized {
///     #[zero_copy(ignore)]
///     field: T,
/// }
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct OtherStruct {
///     #[zero_copy(ignore)]
///     field: Struct<()>,
/// }
///
/// fn assert_zero_sized<T: ZeroSized>() {
///     assert_eq!(size_of::<T>(), 0);
/// }
///
/// assert_zero_sized::<()>();
/// assert_zero_sized::<PhantomData<u32>>();
/// assert_zero_sized::<OtherStruct>();
/// assert_zero_sized::<Struct<OtherStruct>>();
/// ```
pub unsafe trait ZeroSized {}

/// [`ZeroCopy`] implementation for `Wrapping<T>`.
///
/// # Examples
///
/// ```
/// use std::num::Wrapping;
///
/// use musli_zerocopy::{buf, Ref, ZeroCopy};
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Struct {
///     field: Wrapping<u32>,
/// }
///
/// let zero = u32::to_ne_bytes(0);
/// let zero = buf::aligned_buf::<u32>(&zero);
/// let one = u32::to_ne_bytes(1);
/// let one = buf::aligned_buf::<u32>(&one);
///
/// let st = zero.load(Ref::<Struct>::zero())?;
/// assert_eq!(st.field.0, 0);
///
/// let st = one.load(Ref::<Struct>::zero())?;
/// assert_eq!(st.field.0, 1);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
// SAFETY: `Wrapping<T>` is repr-transparent.
unsafe impl<T> ZeroSized for Wrapping<T> where T: ZeroSized {}

unsafe impl<T> ZeroCopy for Wrapping<T>
where
    T: Copy + ZeroCopy,
{
    const ANY_BITS: bool = T::ANY_BITS;
    const PADDED: bool = T::PADDED;

    #[inline]
    unsafe fn pad(this: *const Self, padder: &mut Padder<'_, Self>) {
        padder.pad(this.cast::<T>());
    }

    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        validator.validate::<T>()
    }
}

/// `()` can be ignored as a zero-sized field.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct Struct {
///     #[zero_copy(ignore)]
///     field: (),
/// }
/// ```
// SAFETY: `()` is zero-sized.
unsafe impl ZeroSized for () {}

/// `[T; 0]` can be ignored as a zero-sized field.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct Struct<T> {
///     #[zero_copy(ignore)]
///     field: [T; 0],
/// }
/// ```
// SAFETY: `[T; 0]` is zero-sized.
unsafe impl<T> ZeroSized for [T; 0] {}

/// `PhantomData<T>` can be ignored as a zero-sized field.
///
/// # Examples
///
/// ```
/// use std::marker::PhantomData;
/// use musli_zerocopy::ZeroCopy;
///
/// #[derive(ZeroCopy)]
/// #[repr(transparent)]
/// struct Struct<T> {
///     #[zero_copy(ignore)]
///     field: PhantomData<T>,
/// }
/// ```
// SAFETY: `PhantomData<T>` is zero-sized.
unsafe impl<T: ?Sized> ZeroSized for PhantomData<T> {}

/// Trait governing types can be safely coerced into a reference from a buffer.
///
/// It is not recommended to implement this trait manually, instead rely on the
/// [`ZeroCopy`] derive.
///
/// [`ZeroCopy`]: derive@crate::ZeroCopy
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout like `repr(C)` or an enum with
///   `repr(u32)`.
/// * It's size and alignment must be known statically as per [`size_of`] and
///   [`align_of`]. This excludes enums which are `#[repr(C)]` because for
///   example their alignment depends on the range of values they can represent.
///
/// [`size_of`]: core::mem::size_of
///
/// # Notable types which cannot be `ZeroCopy`
///
/// Any type which does not have an explicit representation cannot implement
/// `ZeroCopy`. Most Rust types use the Rust. Or `#[repr(Rust)]`. The Rust as a
/// language is allowed to make arbitrary layout decisions for `#[repr(Rust)]`
/// types.
///
/// The following is a list of common Rust types which *cannot* implements
/// `ZeroCopy`, and the rationale for why:
///
/// * Non-zero sized tuples. Since tuples do not have a stable layout.
/// * `Option<T>` since that is a `#[repr(Rust)]` type, except where [specific
///   representation guarantees] are made such as with `Option<NonZero*>` types.
///
/// [specific representation guarantees]:
///     https://doc.rust-lang.org/std/option/index.html#representation
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{OwnedBuf, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C)]
/// struct Custom { field: u32, #[zero_copy(ignore)] ignore: () }
///
/// let mut buf = OwnedBuf::new();
/// let ptr = buf.store(&Custom { field: 42, ignore: () });
/// let buf = buf.into_aligned();
/// assert_eq!(buf.load(ptr)?, &Custom { field: 42, ignore: () });
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub unsafe trait ZeroCopy: Sized {
    /// Indicates if the type can inhabit all possible bit patterns within its
    /// `size_of::<Self>()` bytes.
    const ANY_BITS: bool;

    /// Indicates that a type needs padding in case it is stored in an array
    /// that is aligned to `align_of::<Self>()`.
    const PADDED: bool;

    /// Mark padding for the current type.
    ///
    /// The `this` receiver takes the current type as pointer instead of a
    /// reference, because it might not be aligned in the case of packed types.
    ///
    /// # Safety
    ///
    /// The implementor is responsible for ensuring that every field is provided
    /// to `padder`, including potentially hidden ones.
    unsafe fn pad(this: *const Self, padder: &mut Padder<'_, Self>);

    /// Validate the current type.
    ///
    /// # Safety
    ///
    /// This assumes that the provided validator is wrapping a buffer that is
    /// appropriately sized and aligned.
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error>;
}

unsafe impl UnsizedZeroCopy for str {
    const ALIGN: usize = align_of::<u8>();
    const SIZE: usize = size_of::<u8>();

    #[inline]
    fn size(&self) -> usize {
        <str>::len(self)
    }

    #[inline]
    fn bytes(&self) -> usize {
        size_of_val(self)
    }

    #[inline]
    unsafe fn store(&self, buf: &mut BufMut<'_>) {
        buf.store_bytes(self.as_bytes());
    }

    #[inline]
    unsafe fn coerce(buf: *const u8, size: usize) -> Result<*const Self, Error> {
        let buf = slice::from_raw_parts(buf, size);
        Ok(str::from_utf8(buf).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))?)
    }

    #[inline]
    unsafe fn coerce_mut(buf: *mut u8, size: usize) -> Result<*mut Self, Error> {
        let buf = slice::from_raw_parts_mut(buf, size);
        Ok(str::from_utf8_mut(buf).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))?)
    }
}

/// Macro to implement `UnsizedZeroCopy`.
///
/// Its requirements are the following:
/// * Can only be implemented for types which can inhabit any bit-pattern.
/// * Must only be implemented for types which are not padded (as per
///   [`ZeroCopy::PADDED`]).
macro_rules! impl_unsized_primitive {
    ({$($param:ident)?}, $ty:ty, $example_ty:ty, $example:expr $(, $import:path)?) => {
        impl $(<$param>)* self::sealed::Sealed for [$ty] {}

        #[doc = concat!("[`UnsizedZeroCopy`] implementation for [", stringify!($ty), "]")]
        ///
        /// # Examples
        ///
        /// ```
        $(#[doc = concat!("use ", stringify!($import), ";")])*
        /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
        /// use musli_zerocopy::pointer::Unsized;
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        #[doc = concat!("struct Custom", stringify!($(<$param>)*) ," { field: Unsized<[", stringify!($ty) ,"]> }")]
        ///
        /// let mut buf = OwnedBuf::new();
        #[doc = concat!("let unsize: Unsized<[", stringify!($example_ty), "]> = buf.store_unsized(&", stringify!($example), ");")]
        /// let buf = buf.into_aligned();
        #[doc = concat!("assert_eq!(buf.load(unsize)?, &", stringify!($example), ");")]
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        #[allow(rustdoc::invalid_html_tags)]
        unsafe impl $(<$param>)* UnsizedZeroCopy for [$ty] {
            const ALIGN: usize = align_of::<$ty>();
            const SIZE: usize = size_of::<$ty>();

            #[inline]
            fn size(&self) -> usize {
                self.len()
            }

            #[inline]
            fn bytes(&self) -> usize {
                size_of_val(self)
            }

            #[inline]
            unsafe fn store(&self, buf: &mut BufMut<'_>) {
                buf.store_bytes(self);
            }

            #[inline]
            unsafe fn coerce(buf: *const u8, size: usize) -> Result<*const Self, Error> {
                Ok(slice::from_raw_parts(buf.cast(), size))
            }

            #[inline]
            unsafe fn coerce_mut(buf: *mut u8, size: usize) -> Result<*mut Self, Error> {
                Ok(slice::from_raw_parts_mut(buf.cast(), size))
            }
        }
    };
}

impl_unsized_primitive!({}, u8, u8, [u8::MIN, 1, 2, 3, 4, u8::MAX]);
impl_unsized_primitive!({}, i8, i8, [i8::MIN, -1, 2, -3, 4, i8::MAX]);
impl_unsized_primitive!({}, u16, u16, [u16::MIN, 1, 2, 3, 4, u16::MAX]);
impl_unsized_primitive!({}, i16, i16, [i16::MIN, -1, 2, -3, 4, i16::MAX]);
impl_unsized_primitive!({}, u32, u32, [u32::MIN, 1, 2, 3, 4, u32::MAX]);
impl_unsized_primitive!({}, i32, i32, [i32::MIN, -1, 2, -3, 4, i32::MAX]);
impl_unsized_primitive!({}, u64, u64, [u64::MIN, 1, 2, 3, 4, u64::MAX]);
impl_unsized_primitive!({}, i64, i64, [i64::MIN, -1, 2, -3, 4, i64::MAX]);
impl_unsized_primitive!({}, u128, u128, [u128::MIN, 1, 2, 3, 4, u128::MAX]);
impl_unsized_primitive!({}, i128, i128, [i128::MIN, -1, 2, -3, 4, i128::MAX]);
impl_unsized_primitive!({}, (), (), [(), ()]);
impl_unsized_primitive!(
    { T },
    PhantomData<T>,
    PhantomData<u32>,
    [PhantomData, PhantomData],
    std::marker::PhantomData
);

macro_rules! impl_number {
    ($ty:ty) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        /// use std::slice;
        /// use std::mem::size_of;
        /// use musli_zerocopy::{buf, Ref, ZeroCopy};
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($ty), ",")]
        /// }
        ///
        #[doc = concat!("let zero: ", stringify!($ty), " = 0;")]
        #[doc = concat!("let one: ", stringify!($ty), " = 1;")]
        ///
        #[doc = concat!("let zero = ", stringify!($ty), "::to_ne_bytes(0);")]
        #[doc = concat!("let zero = buf::aligned_buf::<", stringify!($ty), ">(&zero);")]
        #[doc = concat!("let one = ", stringify!($ty), "::to_ne_bytes(1);")]
        #[doc = concat!("let one = buf::aligned_buf::<", stringify!($ty), ">(&one);")]
        ///
        /// let st = zero.load(Ref::<Struct>::new(0))?;
        /// assert_eq!(st.field, 0);
        ///
        /// let st = one.load(Ref::<Struct>::new(0))?;
        /// assert_eq!(st.field, 1);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;

            #[inline]
            unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }
        }

        impl Visit for $ty {
            type Target = $ty;

            #[inline]
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

macro_rules! impl_float {
    ($ty:ty) => {
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;

            #[inline]
            unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }
        }

        impl Visit for $ty {
            type Target = $ty;

            #[inline]
            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
            }
        }
    };
}

impl_float!(f32);
impl_float!(f64);

unsafe impl ZeroCopy for char {
    const ANY_BITS: bool = false;
    const PADDED: bool = false;

    #[inline]
    unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        let repr = validator.load_unaligned::<u32>()?;

        if char::try_from(repr).is_err() {
            return Err(Error::new(ErrorKind::IllegalChar { repr }));
        }

        Ok(())
    }
}

impl Visit for char {
    type Target = char;

    #[inline]
    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}

unsafe impl ZeroCopy for bool {
    const ANY_BITS: bool = false;
    const PADDED: bool = false;

    #[inline]
    unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        match validator.byte() {
            0 | 1 => (),
            repr => return Err(Error::new(ErrorKind::IllegalBool { repr })),
        }

        Ok(())
    }
}

impl Visit for bool {
    type Target = bool;

    #[inline]
    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}

macro_rules! impl_nonzero_number {
    ($ty:ident, $inner:ty) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("use std::num::", stringify!($ty), ";")]
        /// use std::slice;
        /// use std::mem::size_of;
        /// use musli_zerocopy::{buf, Ref, ZeroCopy};
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($ty), ",")]
        /// }
        ///
        #[doc = concat!("let zero = ", stringify!($inner), "::to_ne_bytes(0);")]
        #[doc = concat!("let zero = buf::aligned_buf::<", stringify!($ty), ">(&zero);")]
        #[doc = concat!("let one = ", stringify!($inner), "::to_ne_bytes(1);")]
        #[doc = concat!("let one = buf::aligned_buf::<", stringify!($ty), ">(&one);")]
        ///
        /// // Non-zero buffer works as expected.
        /// let st = one.load(Ref::<Struct>::zero())?;
        /// assert_eq!(st.field.get(), 1);
        ///
        /// // Trying to use a zeroed buffer with a non-zero type.
        /// assert!(zero.load(Ref::<Struct>::zero()).is_err());
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for ::core::num::$ty {
            const ANY_BITS: bool = false;
            const PADDED: bool = false;

            #[inline]
            unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
                if validator.load_unaligned::<$inner>()? == 0 {
                    return Err(Error::new(ErrorKind::NonZeroZeroed {
                        range: validator.range::<::core::num::$ty>(),
                    }));
                }

                Ok(())
            }
        }

        impl Visit for ::core::num::$ty {
            type Target = ::core::num::$ty;

            #[inline]
            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
            }
        }

        #[doc = concat!(" [`ZeroCopy`] implementation for `Option<", stringify!($ty), ">`")]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("use std::num::", stringify!($ty), ";")]
        /// use std::slice;
        /// use std::mem::size_of;
        /// use musli_zerocopy::{buf, Ref, ZeroCopy};
        ///
        /// #[derive(ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: Option<", stringify!($ty), ">,")]
        /// }
        ///
        #[doc = concat!("let zero = ", stringify!($inner), "::to_ne_bytes(0);")]
        #[doc = concat!("let zero = buf::aligned_buf::<", stringify!($ty), ">(&zero);")]
        #[doc = concat!("let one = ", stringify!($inner), "::to_ne_bytes(1);")]
        #[doc = concat!("let one = buf::aligned_buf::<", stringify!($ty), ">(&one);")]
        ///
        /// let st = zero.load(Ref::<Struct>::new(0))?;
        /// assert_eq!(st.field, None);
        ///
        /// let st = one.load(Ref::<Struct>::new(0))?;
        #[doc = concat!("assert_eq!(st.field, ", stringify!($ty), "::new(1));")]
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for Option<::core::num::$ty> {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;

            #[inline]
            unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }
        }

        impl Visit for Option<::core::num::$ty> {
            type Target = Option<::core::num::$ty>;

            #[inline]
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
    ($({$($bounds:tt)*},)? $ty:ty, $expr:expr , {$example:ty $(, $import:path)?}) => {
        #[doc = concat!(" [`ZeroCopy`] implementation for `", stringify!($ty), "`")]
        ///
        /// # Examples
        ///
        /// ```
        $(#[doc = concat!("use ", stringify!($import), ";")])*
        /// use musli_zerocopy::{ZeroCopy, OwnedBuf};
        ///
        /// #[derive(Default, Clone, Copy, ZeroCopy)]
        /// #[repr(C)]
        /// struct Struct {
        #[doc = concat!("    field: ", stringify!($example), ",")]
        /// }
        ///
        /// let mut empty = OwnedBuf::new();
        /// let values = [Struct::default(); 100];
        /// let slice = empty.store_slice(&values[..]);
        /// let buf = empty.into_aligned();
        /// assert_eq!(buf.len(), 0);
        ///
        /// let slice = buf.load(slice)?;
        /// assert_eq!(slice.len(), 100);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl $(<$($bounds)*>)* ZeroCopy for $ty {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;

            #[inline]
            unsafe fn pad(_: *const Self, _: &mut Padder<'_, Self>) {
            }

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }
        }

        impl $(<$($bounds)*>)* Visit for $ty {
            type Target = $ty;

            #[inline]
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

/// [`ZeroCopy`] implementation for `[T; 0]`.
///
/// # Examples
///
/// ```
/// use std::mem::align_of;
///
/// use musli_zerocopy::{ZeroCopy, OwnedBuf};
///
/// #[derive(Default, Clone, Copy, ZeroCopy)]
/// #[repr(C)]
/// struct Struct<T> {
///     #[zero_copy(ignore)]
///     field: [T; 0],
/// }
///
/// let mut empty = OwnedBuf::with_alignment::<u128>();
/// let values = [Struct::<u128>::default(); 100];
/// let slice = empty.store_slice(&values[..]);
/// let buf = empty.into_aligned();
/// assert_eq!(buf.len(), 0);
///
/// let slice = buf.load(slice)?;
/// assert_eq!(slice.len(), 100);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
unsafe impl<T, const N: usize> ZeroCopy for [T; N]
where
    T: ZeroCopy,
{
    const ANY_BITS: bool = T::ANY_BITS;
    const PADDED: bool = T::PADDED;

    #[inline]
    unsafe fn pad(this: *const Self, padder: &mut Padder<'_, Self>) {
        if T::PADDED {
            // Cast the array to a pointer of the first element.
            let mut first = this.cast::<T>();

            for _ in 0..N {
                padder.pad::<T>(first);
                first = first.add(1);
            }
        }
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        crate::buf::validate_array::<_, T>(validator, N)?;
        Ok(())
    }
}

impl<T> Visit for [T; 0] {
    type Target = [T; 0];

    #[inline]
    fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        Ok(visitor(self))
    }
}
