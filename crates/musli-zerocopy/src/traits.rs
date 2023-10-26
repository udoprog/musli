//! Traits that apply to types which can safely interact with Müsli's zero copy
//! system.
//!
//! Note that all of these traits are `unsafe`, and require care to implement.
//! Please see their corresponding safety documentation or use the
//! [`ZeroCopy`][derive@crate::ZeroCopy] derive.
//!
//! * [`ZeroCopy`] for types which can safely be coerced from a [`Ref<P>`] to
//!   `&T` or `&mut T`.
//! * [`UnsizedZeroCopy`] for types which can safely be coerced from an
//!   [`Ref<P>`] where `T: ?Sized` to `&T` or `&mut T`.
//! * [`ZeroSized`] for types which can be ignored when deriving
//!   [`ZeroCopy`][derive@crate::ZeroCopy] using `#[zero_copy(ignore)]`.
//!
//! [`Ref<P>`]: crate::pointer::Ref

#![allow(clippy::missing_safety_doc)]

use core::array;
use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::num::Wrapping;
use core::slice;
use core::str;

use crate::buf::{Buf, Padder, Validator, Visit};
use crate::endian::ByteOrder;
use crate::error::{Error, ErrorKind};
use crate::pointer::{Pointee, Size};

mod sealed {
    use crate::ZeroCopy;

    pub trait Sealed {}
    impl Sealed for str {}
    impl<T> Sealed for [T] where T: ZeroCopy {}
}

/// Trait governing which `P` in [`Ref<P>`] where `P: ?Sized` the wrapper can
/// handle.
///
/// We only support slice-like, unaligned unsized types, such as `str` and
/// `[u8]`. We can't support types such as `dyn Debug` because metadata is a
/// vtable which can't be serialized.
///
/// [`Ref<P>`]: crate::pointer::Ref
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
pub unsafe trait UnsizedZeroCopy<P: ?Sized, O>: self::sealed::Sealed
where
    P: Pointee<O>,
{
    /// Alignment of the pointed-to data
    const ALIGN: usize;

    /// If the pointed-to data contains any padding.
    const PADDED: bool;

    /// Return a pointer to the base of the pointed-to value.
    fn as_ptr(&self) -> *const u8;

    /// Metadata associated with the unsized value that is embedded in the
    /// pointer.
    fn metadata(&self) -> P::Metadata;

    /// Apply padding as per the pointed-to value.
    unsafe fn pad(&self, pad: &mut Padder<'_, Self>);

    /// Validate the buffer with the given capacity and return the decoded
    /// metadata.
    unsafe fn validate_unsized<E: ByteOrder>(
        ptr: *const u8,
        len: usize,
        metadata: P::Packed,
    ) -> Result<P::Metadata, Error>;

    /// Construct a wide pointer from a pointer and its associated metadata.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the pointer is valid. The
    /// base pointer `ptr` has to point to a region of memory that is
    /// initialized per `P::Metadata` requirements. Practically that means it's
    /// passed a call to [`validate_unsized()`].
    ///
    /// [`validate_unsized()`]: Self::validate_unsized
    unsafe fn ptr_with_metadata(ptr: *const u8, metadata: P::Metadata) -> *const Self;

    /// Construct a wide mutable pointer from a pointer and its associated
    /// metadata.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the pointer is valid. The
    /// base pointer `ptr` has to point to a region of memory that is
    /// initialized per `P::Metadata` requirements. Practically that means it's
    /// passed a call to [`validate_unsized()`].
    ///
    /// [`validate_unsized()`]: Self::validate_unsized
    unsafe fn ptr_with_metadata_mut(ptr: *mut u8, metadata: P::Metadata) -> *mut Self;
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
    const CAN_SWAP_BYTES: bool = T::CAN_SWAP_BYTES;

    #[inline]
    unsafe fn pad(padder: &mut Padder<'_, Self>) {
        padder.pad::<T>();
    }

    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        validator.validate::<T>()
    }

    #[inline]
    fn swap_bytes<E: ByteOrder>(self) -> Self {
        Wrapping(T::swap_bytes::<E>(self.0))
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
/// Using [`to_bytes`], [`from_bytes`], and [`from_bytes_mut`]:
///
/// [`to_bytes`]: Self::to_bytes
/// [`from_bytes`]: Self::from_bytes
/// [`from_bytes_mut`]: Self::from_bytes_mut
///
/// ```
/// use musli_zerocopy::{buf, ZeroCopy};
///
/// #[derive(ZeroCopy, Debug, PartialEq)]
/// #[repr(C)]
/// struct Weapon {
///     id: u8,
///     damage: u32,
/// }
///
/// let mut weapon = Weapon {
///     id: 1,
///     damage: 42u32,
/// };
///
/// let original = weapon.to_bytes();
///
/// // Make a copy that we can play around with.
/// let mut bytes = buf::aligned_buf::<Weapon>(original).into_owned();
///
/// assert_eq!(weapon.damage, 42);
/// assert_eq!(&weapon, Weapon::from_bytes(&bytes[..])?);
///
/// # #[cfg(target_endian = "little")]
/// assert_eq!(&bytes[..], &[1, 0, 0, 0, 42, 0, 0, 0]);
/// Weapon::from_bytes_mut(&mut bytes[..])?.damage += 10;
/// # #[cfg(target_endian = "little")]
/// assert_eq!(&bytes[..], &[1, 0, 0, 0, 52, 0, 0, 0]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Unsafely access an immutable reference by manually padding the struct using
/// [`initialize_padding()`] and [`to_bytes_unchecked()`]:
///
/// [`initialize_padding()`]: Self::initialize_padding
/// [`to_bytes_unchecked()`]: Self::to_bytes_unchecked
///
/// ```
/// use musli_zerocopy::ZeroCopy;
/// # #[derive(ZeroCopy, Debug, PartialEq)]
/// # #[repr(C)]
/// # struct Weapon { id: u8, damage: u32 }
///
/// let mut weapon = Weapon {
///     id: 1,
///     damage: 42u32,
/// };
///
/// weapon.initialize_padding();
///
/// // SAFETY: Padding for the type has been initialized, and the type has not been moved since it was padded.
/// let bytes = unsafe { weapon.to_bytes_unchecked() };
/// # #[cfg(target_endian = "little")]
/// assert_eq!(bytes, &[1, 0, 0, 0, 42, 0, 0, 0]);
/// assert_eq!(Weapon::from_bytes(&bytes)?, &weapon);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Interacting with an [`OwnedBuf`]:
///
/// [`OwnedBuf`]: crate::buf::OwnedBuf
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
    /// [`size_of::<Self>()`] bytes.
    const ANY_BITS: bool;

    /// Indicates if a type is padded.
    const PADDED: bool;

    /// Indicates if the type has a valid byte-ordered transformation.
    ///
    /// Most notably this is `false` for [`char`].
    const CAN_SWAP_BYTES: bool;

    /// Mark padding for the current type.
    ///
    /// The `this` receiver takes the current type as pointer instead of a
    /// reference, because it might not be aligned in the case of packed types.
    ///
    /// # Safety
    ///
    /// The implementor is responsible for ensuring that every field is provided
    /// to `padder`, including potentially hidden ones.
    #[doc(hidden)]
    unsafe fn pad(padder: &mut Padder<'_, Self>);

    /// Validate the current type.
    ///
    /// # Safety
    ///
    /// This assumes that the provided validator is wrapping a buffer that is
    /// appropriately sized and aligned.
    #[doc(hidden)]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error>;

    /// Ensure that the padding for the current value is initialized.
    ///
    /// This can be used in combination with [`to_bytes_unchecked()`] to relax
    /// the borrowing requirement for [`to_bytes()`], but is `unsafe`.
    ///
    /// See the [type level documentation] for examples.
    ///
    /// [`to_bytes_unchecked()`]: Self::to_bytes_unchecked
    /// [`to_bytes()`]: Self::to_bytes
    /// [type level documentation]: Self
    fn initialize_padding(&mut self) {
        unsafe {
            let ptr = (self as *mut Self).cast::<u8>();

            if Self::PADDED {
                let mut padder = Padder::new(ptr);
                Self::pad(&mut padder);
                padder.remaining();
            }
        }
    }

    /// Convert a reference to a `ZeroCopy` type into bytes.
    ///
    /// This requires mutable access to `self`, since it must call
    /// [`initialize_padding()`] to ensure that the returned buffer is fully
    /// initialized.
    ///
    /// See the [type level documentation] for examples.
    ///
    /// [`initialize_padding()`]: Self::initialize_padding
    /// [type level documentation]: Self
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::ZeroCopy;
    ///
    /// let mut value = 42u32;
    /// assert_eq!(value.to_bytes(), 42u32.to_ne_bytes());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn to_bytes(&mut self) -> &[u8] {
        self.initialize_padding();

        unsafe {
            let ptr = (self as *mut Self).cast::<u8>();
            slice::from_raw_parts(ptr, size_of::<Self>())
        }
    }

    /// Convert a `ZeroCopy` type into bytes.
    ///
    /// This does not require mutable access to `self`, but the caller must
    /// ensure that [`initialize_padding()`] has been called at some point before this
    /// function and that the type that was padded has not been moved.
    ///
    /// See the [type level documentation] for examples.
    ///
    /// [`initialize_padding()`]: Self::initialize_padding
    /// [type level documentation]: Self
    #[inline]
    unsafe fn to_bytes_unchecked(&self) -> &[u8] {
        unsafe {
            let ptr = (self as *const Self).cast::<u8>();
            slice::from_raw_parts(ptr, size_of::<Self>())
        }
    }

    /// Load bytes into a reference of `Self`.
    ///
    /// See the [type level documentation] for examples.
    ///
    /// [type level documentation]: Self
    ///
    /// # Errors
    ///
    /// This will ensure that `bytes` is aligned, appropriately sized, and valid
    /// to inhabit `&Self`. Anything else will cause an [`Error`] detailing why
    /// the conversion failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(&1u32.to_ne_bytes());
    ///
    /// let bytes: &[u8] = &buf[..];
    /// assert_eq!(*u32::from_bytes(&bytes)?, 1);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<&Self, Error> {
        Buf::new(bytes).load_at::<Self>(0)
    }

    /// Load bytes into a mutable reference of `Self`.
    ///
    /// See the [type level documentation] for examples.
    ///
    /// [type level documentation]: Self
    ///
    /// # Errors
    ///
    /// This will ensure that `bytes` is aligned, appropriately sized, and valid
    /// to inhabit `&Self`. Anything else will cause an [`Error`] detailing why
    /// the conversion failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(&1u32.to_ne_bytes());
    ///
    /// *u32::from_bytes_mut(&mut buf[..])? += 10;
    ///
    /// assert_eq!(*u32::from_bytes(&buf[..])?, 11);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Self, Error> {
        Buf::new_mut(bytes).load_at_mut::<Self>(0)
    }

    /// Swap the bytes of `self` using the specified byte ordering to match the
    /// native byte ordering.
    ///
    /// If the specified [`ByteOrder`] matches the current ordering, this is a
    /// no-op.
    ///
    /// This will cause any byte-order sensitive primitives to be converted to
    /// the native byte order.
    ///
    /// # Complex types
    ///
    /// For complex types, this will walk the type hierarchy and swap each
    /// composite field that is apart of that type.
    ///
    /// ```
    /// use musli_zerocopy::{BigEndian, LittleEndian, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Struct {
    ///     number: u32,
    ///     reference: Ref<u32, LittleEndian, usize>,
    /// }
    ///
    /// let st = Struct {
    ///     number: 0x10203040u32.to_le(),
    ///     reference: Ref::new(0x50607080usize.to_le()),
    /// };
    ///
    /// assert_eq!(st.number, 0x10203040u32.to_le());
    /// assert_eq!(st.reference.offset(), 0x50607080usize);
    ///
    /// let st2 = st.swap_bytes::<BigEndian>();
    /// assert_eq!(st2.number, 0x10203040u32.to_be());
    /// assert_eq!(st2.reference.offset(), 0x50607080usize);
    /// ```
    ///
    /// # Safety
    ///
    /// There's nothing fundamentally unsafe about byte swapping, all though it
    /// should be noted that the exact output is not guaranteed to be stable in
    /// case a type cannot be safely byte-swapped. This is the case for
    /// [`char`], since byte swapping one might cause it to inhabit an illegal
    /// bit pattern.
    ///
    /// To test whether a type can be byte swapped, the [`CAN_SWAP_BYTES`]
    /// constant should be advised.
    ///
    /// [`CAN_SWAP_BYTES`]: Self::CAN_SWAP_BYTES
    fn swap_bytes<E: ByteOrder>(self) -> Self;
}

unsafe impl<P: ?Sized, O> UnsizedZeroCopy<P, O> for str
where
    P: Pointee<O, Packed = O, Metadata = usize>,
    O: Size,
{
    const ALIGN: usize = align_of::<u8>();
    const PADDED: bool = false;

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        str::as_ptr(self)
    }

    #[inline]
    fn metadata(&self) -> P::Metadata {
        str::len(self)
    }

    #[inline]
    unsafe fn pad(&self, _: &mut Padder<'_, Self>) {}

    #[inline]
    unsafe fn validate_unsized<E: ByteOrder>(
        ptr: *const u8,
        len: usize,
        metadata: P::Packed,
    ) -> Result<P::Metadata, Error> {
        let metadata = metadata.as_usize::<E>();

        if metadata > len {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: 0..metadata,
                len,
            }));
        };

        let buf = slice::from_raw_parts(ptr, metadata);
        str::from_utf8(buf).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))?;
        Ok(metadata)
    }

    #[inline]
    unsafe fn ptr_with_metadata(ptr: *const u8, metadata: P::Metadata) -> *const Self {
        let slice = slice::from_raw_parts(ptr, metadata);
        str::from_utf8_unchecked(slice)
    }

    #[inline]
    unsafe fn ptr_with_metadata_mut(ptr: *mut u8, metadata: P::Metadata) -> *mut Self {
        let slice = slice::from_raw_parts_mut(ptr, metadata);
        str::from_utf8_unchecked_mut(slice)
    }
}

unsafe impl<T, P: ?Sized, O> UnsizedZeroCopy<P, O> for [T]
where
    T: ZeroCopy,
    P: Pointee<O, Packed = O, Metadata = usize>,
    O: Size,
{
    const ALIGN: usize = align_of::<T>();
    const PADDED: bool = T::PADDED;

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        <[T]>::as_ptr(self).cast()
    }

    #[inline]
    unsafe fn pad(&self, padder: &mut Padder<'_, Self>) {
        for _ in 0..self.len() {
            padder.pad::<T>();
        }
    }

    #[inline]
    fn metadata(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn validate_unsized<E: ByteOrder>(
        buf: *const u8,
        len: usize,
        metadata: P::Packed,
    ) -> Result<P::Metadata, Error> {
        let metadata = metadata.as_usize::<E>();

        let Some(size) = metadata.checked_mul(size_of::<T>()) else {
            return Err(Error::new(ErrorKind::LengthOverflow {
                len: metadata,
                size: size_of::<T>(),
            }));
        };

        if size > len {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: 0..metadata,
                len,
            }));
        };

        if !T::ANY_BITS {
            crate::buf::validate_array::<[T], T>(&mut Validator::new(buf), metadata)?;
        }

        Ok(metadata)
    }

    #[inline]
    unsafe fn ptr_with_metadata(buf: *const u8, metadata: P::Metadata) -> *const Self {
        slice::from_raw_parts(buf.cast(), metadata)
    }

    #[inline]
    unsafe fn ptr_with_metadata_mut(buf: *mut u8, metadata: P::Metadata) -> *mut Self {
        slice::from_raw_parts_mut(buf.cast(), metadata)
    }
}

macro_rules! impl_number {
    ($ty:ty, $from_be:path) => {
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
        /// let st = zero.load(Ref::<Struct>::zero())?;
        /// assert_eq!(st.field, 0);
        ///
        /// let st = one.load(Ref::<Struct>::zero())?;
        /// assert_eq!(st.field, 1);
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;
            const CAN_SWAP_BYTES: bool = true;

            #[inline]
            unsafe fn pad(_: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn swap_bytes<E: ByteOrder>(self) -> Self {
                $from_be(self)
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

impl_number!(usize, E::swap_usize);
impl_number!(isize, E::swap_isize);
impl_number!(u8, core::convert::identity);
impl_number!(u16, E::swap_u16);
impl_number!(u32, E::swap_u32);
impl_number!(u64, E::swap_u64);
impl_number!(u128, E::swap_u128);
impl_number!(i8, core::convert::identity);
impl_number!(i16, E::swap_i16);
impl_number!(i32, E::swap_i32);
impl_number!(i64, E::swap_i64);
impl_number!(i128, E::swap_i128);

macro_rules! impl_float {
    ($ty:ty, $from_fn:path) => {
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;
            const CAN_SWAP_BYTES: bool = true;

            #[inline]
            unsafe fn pad(_: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }

            fn swap_bytes<E: ByteOrder>(self) -> Self {
                $from_fn(self)
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

impl_float!(f32, E::swap_f32);
impl_float!(f64, E::swap_f64);

/// The `ZeroCopy` implementation for `char`.
///
/// Validating this type is byte-order sensitive, since the bit-pattern it
/// inhabits needs to align with the bit-patterns it can inhabit.
unsafe impl ZeroCopy for char {
    const ANY_BITS: bool = false;
    const PADDED: bool = false;
    const CAN_SWAP_BYTES: bool = false;

    #[inline]
    unsafe fn pad(_: &mut Padder<'_, Self>) {}

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        let repr = validator.load_unaligned::<u32>()?;

        if char::try_from(repr).is_err() {
            return Err(Error::new(ErrorKind::IllegalChar { repr }));
        }

        Ok(())
    }

    #[inline]
    fn swap_bytes<E: ByteOrder>(self) -> Self {
        self
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
    const CAN_SWAP_BYTES: bool = true;

    #[inline]
    unsafe fn pad(_: &mut Padder<'_, Self>) {}

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        match validator.byte() {
            0 | 1 => (),
            repr => return Err(Error::new(ErrorKind::IllegalBool { repr })),
        }

        Ok(())
    }

    #[inline]
    fn swap_bytes<E: ByteOrder>(self) -> Self {
        self
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
            const CAN_SWAP_BYTES: bool = true;

            #[inline]
            unsafe fn pad(_: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
                // NB: A zeroed bit-pattern is byte-order independent.
                if validator.load_unaligned::<$inner>()? == 0 {
                    return Err(Error::new(ErrorKind::NonZeroZeroed {
                        range: validator.range::<::core::num::$ty>(),
                    }));
                }

                Ok(())
            }

            #[inline]
            fn swap_bytes<E: ByteOrder>(self) -> Self {
                // SAFETY: a value inhabiting zero is byte-order independent.
                unsafe {
                    ::core::num::$ty::new_unchecked(<$inner as ZeroCopy>::swap_bytes::<E>(
                        self.get(),
                    ))
                }
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
        /// let st = zero.load(Ref::<Struct>::zero())?;
        /// assert_eq!(st.field, None);
        ///
        /// let st = one.load(Ref::<Struct>::zero())?;
        #[doc = concat!("assert_eq!(st.field, ", stringify!($ty), "::new(1));")]
        /// # Ok::<_, musli_zerocopy::Error>(())
        /// ```
        unsafe impl ZeroCopy for Option<::core::num::$ty> {
            const ANY_BITS: bool = true;
            const PADDED: bool = false;
            const CAN_SWAP_BYTES: bool = true;

            #[inline]
            unsafe fn pad(_: &mut Padder<'_, Self>) {}

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn swap_bytes<E: ByteOrder>(self) -> Self {
                // SAFETY: All bit-patterns are habitable, zero we can rely on
                // byte-order conversion from the inner type.
                unsafe {
                    transmute(<$inner as ZeroCopy>::swap_bytes::<E>(
                        transmute::<_, $inner>(self),
                    ))
                }
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
        /// let slice = empty.store_unsized(&values[..]);
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
            const CAN_SWAP_BYTES: bool = true;

            #[inline]
            unsafe fn pad(_: &mut Padder<'_, Self>) {
            }

            #[inline]
            unsafe fn validate(_: &mut Validator<'_, Self>) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn swap_bytes<E: ByteOrder>(self) -> Self {
                self
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
/// let slice = empty.store_unsized(&values[..]);
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
    const CAN_SWAP_BYTES: bool = T::CAN_SWAP_BYTES;

    #[inline]
    unsafe fn pad(padder: &mut Padder<'_, Self>) {
        for _ in 0..N {
            padder.pad::<T>();
        }
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), Error> {
        crate::buf::validate_array::<_, T>(validator, N)?;
        Ok(())
    }

    #[inline]
    fn swap_bytes<E: ByteOrder>(self) -> Self {
        let mut iter = self.into_iter();
        array::from_fn(move |_| T::swap_bytes::<E>(iter.next().unwrap()))
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
