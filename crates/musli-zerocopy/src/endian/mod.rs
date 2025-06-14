//! Marker types which define a [`ByteOrder`] to use.

/// A macro that picks which `$expr` to evaluate to based on if the current
/// `#[cfg(target_endian = "..")]` matches `$endian` and optionally
/// `#[cfg(target_pointer_width = "..")]` matches `$pointer_width`.
///
/// A fallback branch is supported with `_ => $expr`.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::endian;
///
/// // Evaluates to 1 on little-endian and 2 on big-endian platforms.
/// let value = endian::pick!("little" => 1, "big" => 2);
/// // Evaluates to 1 on little-endian 64-bit and 2 on other platforms.
/// let value_64 = endian::pick!("little" / "64" => 1, _ => 2);
/// ```
///
/// Bigger example showcasing an archive header with the same data using two
/// headers, and how [`pick!`] can be used to pick between them.
///
/// ```no_run
/// use musli_zerocopy::{endian, ByteOrder, Endian, Ref, ZeroCopy};
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Header {
///     big: Ref<Data<endian::Big>, endian::Big>,
///     little: Ref<Data<endian::Little>, endian::Little>,
/// }
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Data<E = endian::Native>
/// where
///     E: ByteOrder,
/// {
///     name: Ref<str, E>,
///     age: Endian<u32, E>,
/// }
///
/// let header: Header = todo!();
/// let data: Ref<Data> = endian::pick!("big" => header.big, "little" => header.little);
/// ```
///
/// Note that this evaluates to a private type named `UnsupportedEndian` in case
/// the current endianness is not covered:
///
/// ```compile_fail
/// #[cfg(target_endian = "little")]
/// let data: u32 = endian::pick!("big" => 1u32);
/// #[cfg(target_endian = "big")]
/// let data: u32 = endian::pick!("little" => 1u32);
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __pick {
    ($($endian:literal $(/ $pointer_width:literal)? => $expr:expr),+ $(, _ => $fallback:expr)? $(,)?) => {
        match () {
            $(
                #[cfg(all(target_endian = $endian $(, target_pointer_width = $pointer_width)*))]
                () => $expr,
            )*
            #[cfg(not(any($(all(target_endian = $endian $(, target_pointer_width = $pointer_width)*)),*)))]
            () => $crate::__pick_fallback!($($fallback)*)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __pick_fallback {
    () => {
        struct UnsupportedEndian;
        UnsupportedEndian
    };

    ($expr:expr) => {
        $expr
    };
}

/// A macro that matches `$expr` to its associated `$pat` if the current
/// `#[cfg(target_endian = "..")]` matches `$endian` and optionally
/// `#[cfg(target_pointer_width = "..")]` matches `$pointer_width`.
///
/// Note that if running on a platform which is not covered, the result will
/// always be `false`:
///
/// ```
/// use musli_zerocopy::endian;
///
/// enum Enum { First, Second }
///
/// let e = endian::pick!("little" => Enum::First, "big" => Enum::Second);
/// let cond: u32 = 1;
///
/// #[cfg(target_endian = "little")]
/// {
///     assert!(endian::matches!(e, "little" => Enum::First | Enum::Second));
///     assert!(!endian::matches!(e, "big" => Enum::Second));
///     assert!(!endian::matches!(e, "little" => Enum::First if cond == 2, "big" => Enum::Second));
/// }
///
/// #[cfg(target_endian = "big")]
/// {
///     assert!(endian::matches!(e, "big" => Enum::First | Enum::Second));
///     assert!(!endian::matches!(e, "little" => Enum::First));
///     assert!(!endian::matches!(e, "big" => Enum::Second if cond == 2, "little" => Enum::First));
/// }
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __matches {
    ($expr:expr, $($endian:literal $(/ $pointer_width:literal)? => $pat:pat_param $(| $pat_second:pat_param)* $(if $cond:expr)?),+ $(,)?) => {
        match $expr {
            $(
                #[cfg(all(target_endian = $endian $(, target_pointer_width = $pointer_width)*))]
                $pat $(| $pat_second)* $(if $cond)* => true,
            )*
            _ => false,
        }
    };
}

#[doc(inline)]
pub use __pick as pick;

#[doc(inline)]
pub use __matches as matches;

#[doc(inline)]
pub use self::endian::Endian;
mod endian;

/// Alias for the native endian [`ByteOrder`].
#[cfg(target_endian = "little")]
pub type Native = Little;

/// Alias for the native endian [`ByteOrder`].
#[cfg(target_endian = "big")]
pub type Native = Big;

/// Alias for the opposite endian [`ByteOrder`].
#[cfg(target_endian = "little")]
pub type Other = Big;

/// Alias for the opposite endian [`ByteOrder`].
#[cfg(target_endian = "big")]
pub type Other = Little;

/// Marker type indicating that the big endian [`ByteOrder`] is in use.
#[non_exhaustive]
pub struct Big;

/// Marker type indicating that the little endian [`ByteOrder`] is in use.
#[non_exhaustive]
pub struct Little;

use crate::ZeroCopy;

/// Convert the value `T` from [`Big`] to [`Native`] endian.
///
/// This ignores types which has [`ZeroCopy::CAN_SWAP_BYTES`] set to `false`,
/// such as [`char`]. Such values will simply pass through.
///
/// Swapping the bytes of a type which explicitly records its own byte order
/// like [`Ref<T>`] is a no-op.
///
/// [`Ref<T>`]: crate::Ref
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{endian, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C)]
/// struct Struct {
///     c: char,
///     bits32: u32,
///     bits64: u64,
/// }
///
/// let st = endian::from_be(Struct {
///     c: 'a',
///     bits32: 0x10203040u32.to_be(),
///     bits64: 0x5060708090a0b0c0u64.to_be(),
/// });
///
/// assert_eq!(st, Struct {
///     c: 'a',
///     bits32: 0x10203040,
///     bits64: 0x5060708090a0b0c0,
/// });
/// ```
pub fn from_be<T>(value: T) -> T
where
    T: ZeroCopy,
{
    from_endian::<_, Big>(value)
}

/// Convert the value `T` from [`Little`] to [`Native`] endian.
///
/// This ignores types which has [`ZeroCopy::CAN_SWAP_BYTES`] set to `false`,
/// such as [`char`]. Such values will simply pass through.
///
/// Swapping the bytes of a type which explicitly records its own byte order
/// like [`Ref<T>`] is a no-op.
///
/// [`Ref<T>`]: crate::Ref
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{endian, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C)]
/// struct Struct {
///     c: char,
///     bits32: u32,
///     bits64: u64,
/// }
///
/// let st = endian::from_le(Struct {
///     c: 'a',
///     bits32: 0x10203040u32.to_le(),
///     bits64: 0x5060708090a0b0c0u64.to_le(),
/// });
///
/// assert_eq!(st, Struct {
///     c: 'a',
///     bits32: 0x10203040,
///     bits64: 0x5060708090a0b0c0,
/// });
/// ```
#[inline]
pub fn from_le<T>(value: T) -> T
where
    T: ZeroCopy,
{
    from_endian::<_, Little>(value)
}

/// Convert the value `T` from the specified [`ByteOrder`] `E` to [`Native`]
/// endian.
///
/// This ignores types which has [`ZeroCopy::CAN_SWAP_BYTES`] set to `false`,
/// such as [`char`]. Such values will simply pass through.
///
/// Swapping the bytes of a type which explicitly records its own byte order
/// like [`Ref<T>`] is a no-op.
///
/// [`Ref<T>`]: crate::Ref
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{endian, ZeroCopy};
///
/// #[derive(Debug, PartialEq, ZeroCopy)]
/// #[repr(C)]
/// struct Struct {
///     c: char,
///     bits32: u32,
///     bits64: u64,
/// }
///
/// let st = endian::from_endian::<_, endian::Big>(Struct {
///     c: 'a',
///     bits32: 0x10203040u32.to_be(),
///     bits64: 0x5060708090a0b0c0u64.to_be(),
/// });
///
/// assert_eq!(st, Struct {
///     c: 'a',
///     bits32: 0x10203040,
///     bits64: 0x5060708090a0b0c0,
/// });
/// ```
#[inline]
pub fn from_endian<T, E>(value: T) -> T
where
    T: ZeroCopy,
    E: ByteOrder,
{
    value.swap_bytes::<E>().swap_bytes::<Native>()
}

mod sealed {
    use super::{Big, Little};

    pub trait Sealed {}

    impl Sealed for Big {}
    impl Sealed for Little {}
}

/// Defines a byte order to use.
///
/// This trait is implemented by two marker types [`Big`] and
/// [`Little`], and its internals are intentionally hidden. Do not attempt
/// to use them yourself.
pub trait ByteOrder: 'static + Sized + self::sealed::Sealed {
    /// Maps the `value` through `map`, unless the current byte order is
    /// [`Native`].
    #[doc(hidden)]
    fn try_map<T, F>(value: T, map: F) -> T
    where
        F: FnOnce(T) -> T;

    /// Swap the bytes for a `usize` with the current byte order.
    #[doc(hidden)]
    fn swap_usize(value: usize) -> usize;

    /// Swap the bytes for a `isize` with the current byte order.
    #[doc(hidden)]
    fn swap_isize(value: isize) -> isize;

    /// Swap the bytes of a `u16` with the current byte order.
    #[doc(hidden)]
    fn swap_u16(value: u16) -> u16;

    /// Swap the bytes of a `i16` with the current byte order.
    #[doc(hidden)]
    fn swap_i16(value: i16) -> i16;

    /// Swap the bytes for a `u32` with the current byte order.
    #[doc(hidden)]
    fn swap_u32(value: u32) -> u32;

    /// Swap the bytes for a `i32` with the current byte order.
    #[doc(hidden)]
    fn swap_i32(value: i32) -> i32;

    /// Swap the bytes for a `u64` with the current byte order.
    #[doc(hidden)]
    fn swap_u64(value: u64) -> u64;

    /// Swap the bytes for a `i64` with the current byte order.
    #[doc(hidden)]
    fn swap_i64(value: i64) -> i64;

    /// Swap the bytes for a `u128` with the current byte order.
    #[doc(hidden)]
    fn swap_u128(value: u128) -> u128;

    /// Swap the bytes for a `i128` with the current byte order.
    #[doc(hidden)]
    fn swap_i128(value: i128) -> i128;

    /// Swap the bytes for a `f32` with the current byte order.
    #[doc(hidden)]
    fn swap_f32(value: f32) -> f32;

    /// Swap the bytes for a `f64` with the current byte order.
    #[doc(hidden)]
    fn swap_f64(value: f64) -> f64;
}

impl ByteOrder for Little {
    #[cfg(target_endian = "little")]
    #[inline(always)]
    fn try_map<T, F>(value: T, _: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        value
    }

    #[cfg(not(target_endian = "little"))]
    #[inline(always)]
    fn try_map<T, F>(value: T, map: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        map(value)
    }

    #[inline]
    fn swap_usize(value: usize) -> usize {
        usize::from_le(value)
    }

    #[inline]
    fn swap_isize(value: isize) -> isize {
        isize::from_le(value)
    }

    #[inline]
    fn swap_u16(value: u16) -> u16 {
        u16::to_le(value)
    }

    #[inline]
    fn swap_i16(value: i16) -> i16 {
        i16::to_le(value)
    }

    #[inline]
    fn swap_u32(value: u32) -> u32 {
        u32::from_le(value)
    }

    #[inline]
    fn swap_i32(value: i32) -> i32 {
        i32::from_le(value)
    }

    #[inline]
    fn swap_u64(value: u64) -> u64 {
        u64::from_le(value)
    }

    #[inline]
    fn swap_i64(value: i64) -> i64 {
        i64::from_le(value)
    }

    #[inline]
    fn swap_u128(value: u128) -> u128 {
        u128::from_le(value)
    }

    #[inline]
    fn swap_i128(value: i128) -> i128 {
        i128::from_le(value)
    }

    #[inline]
    fn swap_f32(value: f32) -> f32 {
        f32::from_bits(u32::from_le(value.to_bits()))
    }

    #[inline]
    fn swap_f64(value: f64) -> f64 {
        f64::from_bits(u64::from_le(value.to_bits()))
    }
}

impl ByteOrder for Big {
    #[cfg(target_endian = "big")]
    #[inline(always)]
    fn try_map<T, F>(value: T, _: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        value
    }

    #[cfg(not(target_endian = "big"))]
    #[inline(always)]
    fn try_map<T, F>(value: T, map: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        map(value)
    }

    #[inline]
    fn swap_usize(value: usize) -> usize {
        usize::from_be(value)
    }

    #[inline]
    fn swap_isize(value: isize) -> isize {
        isize::from_be(value)
    }

    #[inline]
    fn swap_u16(value: u16) -> u16 {
        u16::to_be(value)
    }

    #[inline]
    fn swap_i16(value: i16) -> i16 {
        i16::to_be(value)
    }

    #[inline]
    fn swap_u32(value: u32) -> u32 {
        u32::from_be(value)
    }

    #[inline]
    fn swap_i32(value: i32) -> i32 {
        i32::from_be(value)
    }

    #[inline]
    fn swap_u64(value: u64) -> u64 {
        u64::from_be(value)
    }

    #[inline]
    fn swap_i64(value: i64) -> i64 {
        i64::from_be(value)
    }

    #[inline]
    fn swap_u128(value: u128) -> u128 {
        u128::from_be(value)
    }

    #[inline]
    fn swap_i128(value: i128) -> i128 {
        i128::from_be(value)
    }

    #[inline]
    fn swap_f32(value: f32) -> f32 {
        f32::from_bits(u32::from_be(value.to_bits()))
    }

    #[inline]
    fn swap_f64(value: f64) -> f64 {
        f64::from_bits(u64::from_be(value.to_bits()))
    }
}
