use crate::error::{CoerceError, CoerceErrorKind};
use crate::pointer::Size;

mod sealed {
    pub trait Sealed<U: ?Sized> {}
}

/// Trait to coerce slice metadata, which defines its length.
///
/// Coercing from one kind of slice to another means that the length must be
/// adjusted. We can only perform upwards adjustments such as `[u32]` to `[u16]`
/// since independent of the length of the slice we know that it defines a
/// region of memory which is appropriately sized.
pub trait CoerceSlice<U>
where
    Self: self::sealed::Sealed<U>,
    U: ?Sized,
{
    /// Resize with the given `factor`.
    #[doc(hidden)]
    fn resize<O>(factor: O) -> O
    where
        O: Size;

    /// Try to resize with the given `factor`.
    #[doc(hidden)]
    fn try_resize<O>(factor: O) -> Result<O, CoerceError>
    where
        O: Size;
}

macro_rules! self_impl_inner {
    ($from:ty, {$($to:ty),*}) => {
        $(
            impl self::sealed::Sealed<[$to]> for [$from] {}

            #[doc = concat!("Defines the coercion from `[", stringify!($from) ,"]` to `[", stringify!($to), "]`.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use musli_zerocopy::Ref;
            ///
            #[doc = concat!("let reference: Ref<", stringify!($from), "> = Ref::zero();")]
            #[doc = concat!("let reference2 = reference.coerce::<[", stringify!($from), "]>();")]
            /// assert_eq!(reference2.len(), 1);
            ///
            #[doc = concat!("let reference3 = reference.coerce::<", stringify!($to), ">();")]
            #[doc = concat!("let reference4 = reference2.coerce::<[", stringify!($to), "]>();")]
            /// assert_eq!(reference4.len(), 1);
            /// ```
            impl CoerceSlice<[$to]> for [$from] {
                #[inline]
                fn resize<O: Size>(len: O) -> O {
                    len
                }

                #[inline]
                fn try_resize<O: Size>(len: O) -> Result<O, CoerceError> {
                    Ok(len)
                }
            }
        )*
    }
}

macro_rules! self_impl {
    ([$({$($from:ty),*}),*], [$($to:tt),*]) => {
        $(
            $(
                self_impl_inner!($from, $to);
            )*
        )*
    };
}

macro_rules! coerce_slice_inner {
    ($factor:ident, $value:literal, $from:ty, {$($to:ty),*}) => {
        $(
            impl self::sealed::Sealed<[$to]> for [$from] {}

            #[doc = concat!("Defines the coercion from `[", stringify!($from) ,"]` to `[", stringify!($to), "]`.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use musli_zerocopy::Ref;
            ///
            #[doc = concat!("let reference: Ref<", stringify!($from), "> = Ref::zero();")]
            #[doc = concat!("let reference2 = reference.coerce::<[", stringify!($to), "]>();")]
            #[doc = concat!("assert_eq!(reference2.len(), ", stringify!($value), ");")]
            ///
            #[doc = concat!("let reference: Ref<[", stringify!($from), "]> = Ref::with_metadata(0u32, 5);")]
            #[doc = concat!("let reference2 = reference.coerce::<[", stringify!($to), "]>();")]
            #[doc = concat!("assert_eq!(reference2.len(), 5 * ", stringify!($value), ");")]
            /// ```
            impl CoerceSlice<[$to]> for [$from] {
                #[inline]
                fn resize<O>(len: O) -> O
                where
                    O: Size,
                {
                    len.wrapping_mul(O::$factor)
                }

                #[inline]
                fn try_resize<O>(len: O) -> Result<O, CoerceError>
                where
                    O: Size,
                {
                    let Some(len) = len.checked_mul(O::$factor) else {
                        return Err(CoerceError::new(CoerceErrorKind::SliceLengthOverflow {
                            item: len.as_usize(),
                            len: O::$factor.as_usize(),
                        }));
                    };

                    Ok(len)
                }
            }
        )*
    }
}

macro_rules! coerce_slice {
    ($factor:ident, $value:literal, [$({$($from:ty),*}),*], [$($to:tt),*]) => {
        $(
            $(
                coerce_slice_inner!($factor, $value, $from, $to);
            )*
        )*
    }
}

self_impl! {
    [
        {u8, i8},
        {u16, i16},
        {u32, i32},
        {u64, i64},
        {u128, i128}
    ],
    [
        {u8, i8},
        {u16, i16, [u8; 2], [i8; 2]},
        {u32, i32, [u16; 2], [i16; 2], [u8; 4], [i8; 4]},
        {u64, i64, [u32; 2], [i32; 2], [u16; 4], [i16; 4], [u8; 8], [i8; 8]},
        {u128, i128, [u64; 2], [i64; 2], [u32; 4], [i32; 4], [u16; 8], [i16; 8], [u8; 16], [i8; 16]}
    ]
}

coerce_slice! {
    N2, 2,
    [
        {u16, i16},
        {u32, i32},
        {u64, i64},
        {u128, i128}
    ],
    [
        {u8, i8},
        {u16, i16, [u8; 2], [i8; 2]},
        {u32, i32, [u16; 2], [i16; 2], [u8; 4], [i8; 4]},
        {u64, i64, [u32; 2], [i32; 2], [u16; 4], [i16; 4], [u8; 8], [i8; 8]}
    ]
}

coerce_slice! {
    N4, 4,
    [
        {u32, i32},
        {u64, i64},
        {u128, i128}
    ],
    [
        {u8, i8},
        {u16, i16, [u8; 2], [i8; 2]},
        {u32, i32, [u16; 2], [i16; 2], [u8; 4], [i8; 4]}
    ]
}

coerce_slice! {
    N8, 8,
    [
        {u64, i64},
        {u128, i128}
    ],
    [
        {u8, i8},
        {u16, i16, [u8; 2], [i8; 2]}
    ]
}

coerce_slice! {
    N16, 16,
    [
        {u128, i128}
    ],
    [
        {u8, i8}
    ]
}
