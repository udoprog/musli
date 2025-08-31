use crate::error::CoerceError;
use crate::pointer::{CoerceSlice, Pointee, Size};
use crate::traits::ZeroCopy;

/// A trait indicating that a coercion from `Self` to `U` is correct from a size
/// perspective.
pub trait Coerce<U>
where
    Self: Pointee,
    U: ?Sized + Pointee,
{
    /// Coerce metadata from `Self` to `U`.
    ///
    /// Any overflow will wrap around.
    fn coerce_metadata<O>(metadata: Self::Stored<O>) -> U::Stored<O>
    where
        O: Size;

    /// Try to coerce metadata from `Self` to `U`.
    ///
    /// Any overflow will result in `None`.
    fn try_coerce_metadata<O>(metadata: Self::Stored<O>) -> Result<U::Stored<O>, CoerceError>
    where
        O: Size;
}

/// Defines a coercion from a slice `[T]` to `[U]`.
///
/// Since slices have a length which depends on the exact sizing of `T` and `U`,
/// this conversion is constrained by a special trait [`CoerceSlice<T>`].
impl<T, U> Coerce<[U]> for [T]
where
    T: ZeroCopy,
    U: ZeroCopy,
    [T]: CoerceSlice<[U]>,
{
    #[inline]
    fn coerce_metadata<O>(metadata: O) -> O
    where
        O: Size,
    {
        <[T]>::resize(metadata)
    }

    #[inline]
    fn try_coerce_metadata<O>(metadata: O) -> Result<O, CoerceError>
    where
        O: Size,
    {
        <[T]>::try_resize(metadata)
    }
}

/// Defines the coercion from `str` to `[T]`.
///
/// Broadly speaking, this inherits the coercions which are possible for `[u8]`,
/// which basically limits it to `u8` and `i8` and other single byte types.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<str> = Ref::with_metadata(0u32, 12);
/// let reference2 = reference.coerce::<[u8]>();
/// let reference3 = reference.coerce::<[i8]>();
/// assert_eq!(reference2.len(), 12);
/// assert_eq!(reference3.len(), 12);
/// ```
impl<T> Coerce<[T]> for str
where
    [u8]: CoerceSlice<[T]>,
    T: ZeroCopy,
{
    #[inline]
    fn coerce_metadata<O: Size>(metadata: O) -> O {
        <[u8]>::resize(metadata)
    }

    #[inline]
    fn try_coerce_metadata<O: Size>(metadata: O) -> Result<O, CoerceError> {
        <[u8]>::try_resize(metadata)
    }
}

/// Defines the coercion from `[T]` to `str`.
///
/// Broadly speaking, this inherits the coercions which are possible into `[u8]`,
/// which means any type can be coerced into a `str`.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<[u32]> = Ref::with_metadata(0u32, 12);
/// let reference2 = reference.coerce::<str>();
/// assert_eq!(reference2.len(), 12 * 4);
/// ```
impl<T> Coerce<str> for [T]
where
    [T]: CoerceSlice<[u8]>,
    T: ZeroCopy,
{
    #[inline]
    fn coerce_metadata<O: Size>(metadata: O) -> O {
        <[T]>::resize(metadata)
    }

    #[inline]
    fn try_coerce_metadata<O: Size>(metadata: O) -> Result<O, CoerceError> {
        <[T]>::try_resize(metadata)
    }
}

macro_rules! same_size_inner {
    ($from:ty, {$($to:ty),*}) => {
        $(
            #[doc = concat!("Defines the coercion for `", stringify!($from) ,"` to `", stringify!($to), "`.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use musli_zerocopy::Ref;
            ///
            #[doc = concat!("let reference: Ref<", stringify!($from), "> = Ref::zero();")]
            #[doc = concat!("let reference2 = reference.coerce::<", stringify!($to), ">();")]
            /// assert_eq!(reference.offset(), reference2.offset());
            /// ```
            impl Coerce<$to> for $from {
                #[inline(always)]
                fn coerce_metadata<O: Size>(metadata: ()) -> () {
                    metadata
                }

                #[inline(always)]
                fn try_coerce_metadata<O: Size>(metadata: ()) -> Result<(), CoerceError> {
                    Ok(metadata)
                }
            }
        )*
    }
}

macro_rules! same_size {
    ([$({$($from:ty),*}),*], [$($to:tt),*]) => {
        $(
            $(
                same_size_inner!($from, $to);
            )*
        )*
    };
}

same_size!([{u8, i8}], [{u8, i8}]);
same_size!([{u16, i16}], [{u16, i16, [u8; 2], [i8; 2]}]);
same_size!([{u32, i32}], [{u32, i32, [u16; 2], [i16; 2], [u8; 4], [i8; 4]}]);
same_size!([{u64, i64}], [{u64, i64, [u32; 2], [i32; 2], [u16; 4], [i16; 4], [u8; 8], [i8; 8]}]);
same_size!([{u128, i128}], [{u128, i128, [u64; 2], [i64; 2], [u32; 4], [i32; 4], [u16; 8], [i16; 8], [u8; 16], [i8; 16]}]);

/// Defines the primitive coercion from `T` to `[U]`.
///
/// This coercion results in a single element slice of type `T`, and is largely
/// defined through the help of [`CoerceSlice<T>`].
///
/// Note that coercing from a smaller to a larger type is not possible, since we
/// don't know how many elements of the smaller type is in use:
///
/// ```compile_fail
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<u8> = Ref::zero();
/// let reference2 = reference.coerce::<[u32]>();
/// ```
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<u32> = Ref::zero();
/// let reference2 = reference.coerce::<[u32]>();
/// assert_eq!(reference2.len(), 1);
///
/// let reference: Ref<u64> = Ref::zero();
/// let reference2 = reference.coerce::<[u32]>();
/// assert_eq!(reference2.len(), 2);
///
/// let reference: Ref<u128> = Ref::zero();
/// let reference2 = reference.coerce::<[u32]>();
/// assert_eq!(reference2.len(), 4);
/// ```
impl<T, U> Coerce<[U]> for T
where
    T: ZeroCopy,
    U: ZeroCopy,
    [T]: CoerceSlice<[U]>,
{
    #[inline]
    fn coerce_metadata<O: Size>((): ()) -> O {
        <[T]>::resize(O::ONE)
    }

    #[inline]
    fn try_coerce_metadata<O: Size>((): ()) -> Result<O, CoerceError> {
        <[T]>::try_resize(O::ONE)
    }
}

/// Defines the coercion from `[T; N]` to `[U]`.
///
/// This coercion results in a single element slice of type `T`, and is largely
/// defined through the help of [`CoerceSlice<T>`].
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<[u32; 2]> = Ref::zero();
/// let reference2 = reference.coerce::<[u32]>();
/// assert_eq!(reference2.len(), 2);
///
/// let reference: Ref<[u128; 4]> = Ref::zero();
/// let reference2 = reference.coerce::<[u64]>();
/// assert_eq!(reference2.len(), 8);
/// ```
impl<T, const N: usize, U> Coerce<[U]> for [T; N]
where
    T: ZeroCopy,
    U: ZeroCopy,
    [T]: CoerceSlice<[U]>,
{
    #[inline]
    fn coerce_metadata<O>((): ()) -> <[T] as Pointee>::Stored<O>
    where
        O: Size,
    {
        <[T]>::resize(O::from_usize(N))
    }

    #[inline]
    fn try_coerce_metadata<O>((): ()) -> Result<<[T] as Pointee>::Stored<O>, CoerceError>
    where
        O: Size,
    {
        <[T]>::try_resize(O::try_from_usize(N)?)
    }
}

macro_rules! non_zero_inner {
    ($from:ident, {$($to:ty),*}) => {
        $(
            #[doc = concat!("Defines the coercion for `", stringify!($from) ,"` to `", stringify!($to), "`.")]
            ///
            /// # Examples
            ///
            /// ```
            #[doc = concat!("use std::num::", stringify!($from), ";")]
            ///
            /// use musli_zerocopy::Ref;
            ///
            #[doc = concat!("let reference: Ref<", stringify!($from), "> = Ref::zero();")]
            #[doc = concat!("let reference2 = reference.coerce::<", stringify!($to), ">();")]
            /// assert_eq!(reference.offset(), reference2.offset());
            /// ```
            impl Coerce<$to> for core::num::$from {
                #[inline(always)]
                fn coerce_metadata<O: Size>(metadata: ()) -> () {
                    metadata
                }

                #[inline(always)]
                fn try_coerce_metadata<O: Size>(metadata: ()) -> Result<(), CoerceError> {
                    Ok(metadata)
                }
            }
        )*
    }
}

macro_rules! non_zero {
    ([$({$($from:ident),*}),*], [$($to:tt),*]) => {
        $(
            $(
                non_zero_inner!($from, $to);
            )*
        )*
    };
}

non_zero!([{NonZeroU8, NonZeroI8}], [{u8, i8}]);
non_zero!([{NonZeroU16, NonZeroI16}], [{u16, i16}]);
non_zero!([{NonZeroU32, NonZeroI32}], [{u32, i32}]);
non_zero!([{NonZeroU64, NonZeroI64}], [{u64, i64}]);
non_zero!([{NonZeroU128, NonZeroI128}], [{u128, i128}]);

/// Defines the coercion for `core::num::Wrapping<T>` to `U`.
///
/// This is largely defined by the [`Coerce<T>`] of `T` to `U`.
///
/// # Examples
///
/// ```
/// use std::num::Wrapping;
///
/// use musli_zerocopy::Ref;
///
/// let reference: Ref<Wrapping<i128>> = Ref::zero();
/// let reference2 = reference.coerce::<u128>();
/// let reference3 = reference.coerce::<[u32; 4]>();
/// let reference4 = reference.coerce::<u128>().coerce::<[u32]>();
/// assert_eq!(reference4.len(), 4);
/// ```
impl<T, U> Coerce<U> for core::num::Wrapping<T>
where
    T: Coerce<U>,
    U: ZeroCopy,
    T: ZeroCopy,
{
    #[inline(always)]
    fn coerce_metadata<O: Size>(metadata: ()) {
        metadata
    }

    #[inline(always)]
    fn try_coerce_metadata<O: Size>(metadata: ()) -> Result<(), CoerceError> {
        Ok(metadata)
    }
}
