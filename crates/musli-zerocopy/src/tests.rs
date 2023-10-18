#![allow(clippy::assertions_on_constants)]

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;

use anyhow::Result;

use crate::pointer::Ref;
use crate::{Error, OwnedBuf, ZeroCopy};

#[test]
fn test_ref_to_slice() -> Result<()> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
    #[repr(u32)]
    #[zero_copy(crate)]
    enum InnerEnum {
        None = 0xfffffffe,
        Some(u32),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    pub struct RefToSlice {
        index: Ref<Ref<[u8]>>,
        extra: InnerEnum,
    }

    let mut buf = OwnedBuf::new();

    let index = buf.store_slice(&[1, 2, 3, 4]);
    let index = buf.store(&index);

    let to_slice1 = RefToSlice {
        index,
        extra: InnerEnum::Some(4040),
    };

    let to_slice2 = RefToSlice {
        index,
        extra: InnerEnum::None,
    };

    let to_slice1_ref = buf.store(&to_slice1);
    let to_slice2_ref = buf.store(&to_slice2);

    let buf = buf.into_aligned();

    assert_eq!(buf.load(&to_slice1_ref)?, &to_slice1);
    assert_eq!(buf.load(&to_slice2_ref)?, &to_slice2);
    Ok(())
}

#[test]
fn test_zero_padded() {
    #[derive(ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C, align(128))]
    struct EmptyPadded;

    const _: () = assert!(EmptyPadded::PADDED);

    #[derive(ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct EmptyDefaultAlign;

    const _: () = assert!(!EmptyDefaultAlign::PADDED);
}

#[test]
fn test_inner_padded() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Inner {
        first: u8,
        second: u32,
    }

    const _: () = assert!(Inner::PADDED);

    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u32,
        second: u32,
        third: Inner,
    }

    const _: () = assert!(Element::PADDED);
    Ok(())
}

#[test]
fn test_inner_not_padded() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Inner {
        first: u8,
        second: [u8; 1],
        third: u8,
        fourth: u8,
    }

    const _: () = assert!(!Inner::PADDED);

    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u32,
        second: u32,
        third: Inner,
    }

    const _: () = assert!(!Element::PADDED);
    Ok(())
}

#[test]
fn test_inner_not_padded_by_align() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C, align(128))]
    struct Inner {
        first: u8,
        second: [u8; 1],
        third: u8,
        fourth: u8,
    }

    const _: () = assert!(Inner::PADDED);

    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u32,
        second: u32,
        third: Inner,
    }

    const _: () = assert!(Element::PADDED);
    Ok(())
}

#[test]
fn weird_alignment() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[repr(C, align(128))]
    #[zero_copy(crate)]
    struct WeirdAlignment {
        array: [u32; 3],
        field: u128,
    }

    let weird = WeirdAlignment {
        array: [0xffffffff, 0xffff0000, 0x0000ffff],
        field: 0x0000ffff0000ffff0000ffff0000ffffu128,
    };

    let mut buf = OwnedBuf::with_alignment::<WeirdAlignment>();
    let w = buf.store(&weird);
    let buf = buf.into_aligned();

    assert_eq!(buf.len(), size_of::<WeirdAlignment>());
    assert_eq!(buf.load(w)?, &weird);
    Ok(())
}

#[test]
fn enum_boundaries() -> Result<()> {
    macro_rules! test_case {
        ($name:ident, $repr:ident, $num:ty, $min:literal, $max:literal, $illegal_repr:ident $(,)?) => {{
            #[derive(Debug, PartialEq, ZeroCopy)]
            #[repr($repr)]
            #[zero_copy(crate)]
            enum $name {
                Variant1 = 2,
                Variant2,
                Variant3 = 5,
                Max = $max,
                Min = $min,
                AfterMin,
            }

            assert_eq!($name::Variant1 as $repr, 2);
            assert_eq!($name::Variant2 as $repr, 3);
            assert_eq!($name::Variant3 as $repr, 5);
            assert_eq!($name::Max as $repr, $max);
            assert_eq!($name::Min as $repr, $min);
            assert_eq!($name::AfterMin as $repr, $min + 1);

            let mut buf = OwnedBuf::with_alignment::<$name>();
            let v1 = buf.store(&$name::Variant1);
            let v2 = buf.store(&$name::Variant2);
            let v3 = buf.store(&$name::Variant3);
            let max = buf.store(&$name::Max);
            let min = buf.store(&$name::Min);
            let after_min = buf.store(&$name::AfterMin);
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX - 1)).offset());

            let buf = buf.into_aligned();

            assert_eq!(buf.load(v1)?, &$name::Variant1);
            assert_eq!(buf.load(v2)?, &$name::Variant2);
            assert_eq!(buf.load(v3)?, &$name::Variant3);
            assert_eq!(buf.load(max)?, &$name::Max);
            assert_eq!(buf.load(min)?, &$name::Min);
            assert_eq!(buf.load(after_min)?, &$name::AfterMin);
            assert_eq!(
                buf.load(v4),
                Err(Error::$illegal_repr::<$name>(<$num>::MAX - 1))
            );
        }};
    }

    test_case!(U8, u8, u8, 0, 255u8, __illegal_enum_u8);
    test_case!(U16, u16, u16, 0, 65_535u16, __illegal_enum_u16);
    test_case!(U32, u32, u32, 0, 4_294_967_295u32, __illegal_enum_u32);
    test_case!(
        U64,
        u64,
        u64,
        0,
        18_446_744_073_709_551_615u64,
        __illegal_enum_u64
    );
    // nightly: feature(repr128)
    #[cfg(feature = "nightly")]
    test_case!(
        U128,
        u128,
        u128,
        0u128,
        340_282_366_920_938_463_463_374_607_431_768_211_455u128
        __illegal_enum_u128,
    );
    test_case!(I8, i8, i8, -128i8, 127i8, __illegal_enum_i8);
    test_case!(I16, i16, i16, -32_768i16, 32_767i16, __illegal_enum_i16);
    test_case!(
        I32,
        i32,
        i32,
        -2_147_483_648i32,
        2_147_483_647i32,
        __illegal_enum_i32
    );
    test_case!(
        I64,
        i64,
        i64,
        -9_223_372_036_854_775_808i64,
        9_223_372_036_854_775_807i64,
        __illegal_enum_i64,
    );
    // nightly: feature(repr128)
    #[cfg(feature = "nightly")]
    test_case!(
        I128,
        i128,
        i128,
        -170_141_183_460_469_231_731_687_303_715_884_105_728i128,
        170_141_183_460_469_231_731_687_303_715_884_105_727i128,
        __illegal_enum_i128,
    );
    Ok(())
}

#[test]
fn test_signed_wraparound() -> Result<()> {
    macro_rules! test_case {
        ($name:ident, $repr:ident, $num:ty, $illegal_repr:ident $(,)?) => {{
            #[derive(Debug, PartialEq, ZeroCopy)]
            #[repr($repr)]
            #[zero_copy(crate)]
            enum $name {
                MinusOne = -1,
                Zero,
                One,
            }

            assert_eq!($name::MinusOne as $repr, -1);
            assert_eq!($name::Zero as $repr, 0);
            assert_eq!($name::One as $repr, 1);

            let mut buf = OwnedBuf::with_alignment::<$name>();
            let minus_one = buf.store(&$name::MinusOne);
            let zero = buf.store(&$name::Zero);
            let one = buf.store(&$name::One);
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX)).offset());

            let buf = buf.into_aligned();

            assert_eq!(buf.load(minus_one)?, &$name::MinusOne);
            assert_eq!(buf.load(zero)?, &$name::Zero);
            assert_eq!(buf.load(one)?, &$name::One);
            assert_eq!(
                buf.load(v4),
                Err(Error::$illegal_repr::<$name>(<$num>::MAX))
            );
        }};
    }

    test_case!(I8, i8, i8, __illegal_enum_i8);
    test_case!(I16, i16, i16, __illegal_enum_i16);
    test_case!(I32, i32, i32, __illegal_enum_i32);
    test_case!(I64, i64, i64, __illegal_enum_i64);
    // nightly: feature(repr128)
    #[cfg(feature = "nightly")]
    test_case!(I128, i128, i128, __illegal_enum_i128);
    Ok(())
}

#[test]
fn test_neg0() -> Result<()> {
    macro_rules! test_case {
        ($name:ident, $repr:ident, $num:ty, $illegal_repr:ident $(,)?) => {{
            #[derive(Debug, PartialEq, ZeroCopy)]
            #[repr($repr)]
            #[zero_copy(crate)]
            enum $name {
                MinusOne = -1,
                Neg0 = -0,
                One,
            }

            assert_eq!($name::MinusOne as $repr, -1);
            assert_eq!($name::Neg0 as $repr, 0);
            assert_eq!($name::One as $repr, 1);

            let mut buf = OwnedBuf::with_alignment::<$name>();
            let minus_one = buf.store(&$name::MinusOne);
            let neg0 = buf.store(&$name::Neg0);
            let one = buf.store(&$name::One);
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX)).offset());

            let buf = buf.into_aligned();

            assert_eq!(buf.load(minus_one)?, &$name::MinusOne);
            assert_eq!(buf.load(neg0)?, &$name::Neg0);
            assert_eq!(buf.load(one)?, &$name::One);
            assert_eq!(
                buf.load(v4),
                Err(Error::$illegal_repr::<$name>(<$num>::MAX))
            );
        }};
    }

    test_case!(I8, i8, i8, __illegal_enum_i8);
    test_case!(I16, i16, i16, __illegal_enum_i16);
    test_case!(I32, i32, i32, __illegal_enum_i32);
    test_case!(I64, i64, i64, __illegal_enum_i64);
    // nightly: feature(repr128)
    #[cfg(feature = "nightly")]
    test_case!(I128, i128, i128, __illegal_enum_i128);
    Ok(())
}

#[test]
fn test_needs_padding() -> Result<()> {
    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct Zst {}

    const _: () = assert!(!Zst::PADDED);

    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct SingleField {
        not_padded: u32,
    }

    const _: () = assert!(!SingleField::PADDED);

    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct MightPad {
        might_pad: [u32; 4],
    }

    const _: () = assert!(!MightPad::PADDED);
    Ok(())
}

#[test]
fn test_enum_with_fields() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[repr(u8)]
    #[zero_copy(crate)]
    enum Types {
        Variant {
            field: u32,
            field2: u64,
        },
        Variant2(u32),
        Variant3,
        Empty {
            #[zero_copy(ignore)]
            empty: PhantomData<u64>,
        },
    }

    let mut buf = OwnedBuf::new();
    let variant = buf.store(&Types::Variant {
        field: 10,
        field2: 20,
    });
    let variant2 = buf.store(&Types::Variant2(40));
    let variant3 = buf.store(&Types::Variant3);
    let empty = buf.store(&Types::Empty { empty: PhantomData });

    let buf = buf.into_aligned();

    assert_eq!(
        buf.load(variant)?,
        &Types::Variant {
            field: 10,
            field2: 20
        }
    );

    assert_eq!(buf.load(variant2)?, &Types::Variant2(40));
    assert_eq!(buf.load(variant3)?, &Types::Variant3);
    assert_eq!(buf.load(empty)?, &Types::Empty { empty: PhantomData });
    Ok(())
}

#[test]
fn validate_packed() -> Result<()> {
    use core::num::NonZeroU64;

    #[derive(ZeroCopy)]
    #[repr(C, packed)]
    #[zero_copy(crate)]
    struct Packed {
        field: u32,
        field2: NonZeroU64,
    }

    assert_eq!(size_of::<Packed>(), 12);
    assert_eq!(align_of::<Packed>(), 1);

    let mut buf = OwnedBuf::new();

    buf.store(&Packed {
        field: 42,
        field2: NonZeroU64::new(84).unwrap(),
    });

    let buf = buf.into_aligned();

    let mut v = buf.validate_struct::<Packed>()?;

    // SAFETY: We're only validating fields we know are
    // part of the struct, and do not go beyond. We're
    // also making sure not to construct reference to
    // the fields which would be an error for a packed struct.
    unsafe {
        v.validate_with::<u32>(1)?;
        v.validate_with::<NonZeroU64>(1)?;
    }

    Ok(())
}

#[cfg(test)]
mod primitive_slices {
    /// Macro to implement `UnsizedZeroCopy`.
    ///
    /// Its requirements are the following:
    /// * Can only be implemented for types which can inhabit any bit-pattern.
    /// * Must only be implemented for types which are not padded (as per
    ///   [`ZeroCopy::PADDED`]).
    macro_rules! test_case {
        ({$($param:ident)?}, $ty:ident, $example:expr) => {
            #[test]
            fn $ty() -> Result<(), crate::Error> {
                use crate::{OwnedBuf, Ref, ZeroCopy};

                #[derive(ZeroCopy)]
                #[repr(C)]
                #[zero_copy(crate)]
                struct Custom $(<$param>)* { field: Ref<[$ty]> }

                let mut buf = OwnedBuf::new();
                let slice: Ref<[$ty]> = buf.store_slice(&$example);
                let buf = buf.into_aligned();
                assert_eq!(buf.load(slice)?, &$example);
                Ok(())
            }
        };
    }

    test_case!({}, u8, [u8::MIN, 1, 2, 3, 4, u8::MAX]);
    test_case!({}, i8, [i8::MIN, -1, 2, -3, 4, i8::MAX]);
    test_case!({}, u16, [u16::MIN, 1, 2, 3, 4, u16::MAX]);
    test_case!({}, i16, [i16::MIN, -1, 2, -3, 4, i16::MAX]);
    test_case!({}, u32, [u32::MIN, 1, 2, 3, 4, u32::MAX]);
    test_case!({}, i32, [i32::MIN, -1, 2, -3, 4, i32::MAX]);
    test_case!({}, u64, [u64::MIN, 1, 2, 3, 4, u64::MAX]);
    test_case!({}, i64, [i64::MIN, -1, 2, -3, 4, i64::MAX]);
    test_case!({}, u128, [u128::MIN, 1, 2, 3, 4, u128::MAX]);
    test_case!({}, i128, [i128::MIN, -1, 2, -3, 4, i128::MAX]);
}

#[cfg(test)]
mod nonzero_slices {
    macro_rules! test_case {
        ($name:ident, $ty:ident, $example:expr $(, $import:path)?) => {
            #[test]
            fn $name() -> Result<(), crate::Error> {
                $(use $import;)*
                use crate::{OwnedBuf, Ref, ZeroCopy};

                #[derive(ZeroCopy)]
                #[repr(C)]
                #[zero_copy(crate)]
                struct Custom { field: Ref<[$ty]> }

                let mut buf = OwnedBuf::new();
                let slice: Ref<[$ty]> = buf.store_slice(&$example).cast::<[$ty]>();
                let buf = buf.into_aligned();
                let example: &[$ty] = unsafe { core::mem::transmute(&$example[..]) };
                assert_eq!(buf.load(slice)?, example);
                Ok(())
            }
        };
    }

    macro_rules! error_case {
        ($name:ident, $ty:ident, $example:expr $(, $import:path)?) => {
            #[test]
            fn $name() -> Result<(), crate::Error> {
                $(use $import;)*
                use crate::{OwnedBuf, Ref, ZeroCopy};

                #[derive(ZeroCopy)]
                #[repr(C)]
                #[zero_copy(crate)]
                struct Custom { field: Ref<[$ty]> }

                let mut buf = OwnedBuf::new();
                let slice: Ref<[$ty]> = buf.store_slice(&$example).cast::<[$ty]>();
                let buf = buf.into_aligned();
                assert!(buf.load(slice).is_err());
                Ok(())
            }
        };
    }

    test_case!(
        non_zero_u8,
        NonZeroU8,
        [1, 2, 3, 4, u8::MAX],
        core::num::NonZeroU8
    );
    test_case!(
        non_zero_i8,
        NonZeroI8,
        [i8::MIN, -1, 2, -3, 4, i8::MAX],
        core::num::NonZeroI8
    );
    test_case!(
        non_zero_u16,
        NonZeroU16,
        [1, 2, 3, 4, u16::MAX],
        core::num::NonZeroU16
    );
    test_case!(
        non_zero_i16,
        NonZeroI16,
        [-1, 2, -3, 4, i16::MAX],
        core::num::NonZeroI16
    );
    test_case!(
        non_zero_u32,
        NonZeroU32,
        [1, 2, 3, 4, u32::MAX],
        core::num::NonZeroU32
    );
    test_case!(
        non_zero_i32,
        NonZeroI32,
        [i32::MIN, -1, 2, -3, 4, i32::MAX],
        core::num::NonZeroI32
    );
    test_case!(
        non_zero_u64,
        NonZeroU64,
        [1, 2, 3, 4, u64::MAX],
        core::num::NonZeroU64
    );
    test_case!(
        non_zero_i64,
        NonZeroI64,
        [i64::MIN, -1, 2, -3, 4, i64::MAX],
        core::num::NonZeroI64
    );
    test_case!(
        non_zero_u128,
        NonZeroU128,
        [1, 2, 3, 4, u128::MAX],
        core::num::NonZeroU128
    );
    test_case!(
        non_zero_i128,
        NonZeroI128,
        [i128::MIN, -1, 2, -3, 4, i128::MAX],
        core::num::NonZeroI128
    );

    error_case!(
        zero_non_zero_u8,
        NonZeroU8,
        [u8::MIN, 1, 2, 3, 4, u8::MAX],
        core::num::NonZeroU8
    );
    error_case!(
        zero_non_zero_u16,
        NonZeroU16,
        [u16::MIN, 1, 2, 3, 4, u16::MAX],
        core::num::NonZeroU16
    );
    error_case!(
        zero_non_zero_u32,
        NonZeroU32,
        [u32::MIN, 1, 2, 3, 4, u32::MAX],
        core::num::NonZeroU32
    );
    error_case!(
        zero_non_zero_u64,
        NonZeroU64,
        [u64::MIN, 1, 2, 3, 4, u64::MAX],
        core::num::NonZeroU64
    );
    error_case!(
        zero_non_zero_u128,
        NonZeroU128,
        [u128::MIN, 1, 2, 3, 4, u128::MAX],
        core::num::NonZeroU128
    );
}

#[test]
fn padded_enum() -> Result<()> {
    // We add a padded interior struct.
    #[derive(Debug, Clone, Copy, PartialEq, ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct InnerStruct(u32, u8);

    #[derive(Debug, Clone, Copy, PartialEq, ZeroCopy)]
    #[repr(u8)]
    #[zero_copy(crate)]
    enum Enum {
        Variant1,
        Variant2(u32),
        Variant3(InnerStruct, u64),
    }

    // In another test, we try to store the enum inside of a packed struct,
    // which will force us to assert that accessing any enum data during
    // validation only uses unaligned loads.
    #[derive(ZeroCopy)]
    #[repr(C, packed)]
    #[zero_copy(crate)]
    struct PackedStruct(u16, Enum);

    let mut buf = OwnedBuf::with_alignment::<Enum>();

    macro_rules! test_case {
        ($variant:expr, $slice:expr) => {{
            let slice = $slice;

            assert_eq!(size_of::<Enum>(), slice.len());

            buf.clear();
            let r = buf.store(&$variant);
            assert_eq!(buf.as_slice(), slice);
            assert_eq!(buf.load(r)?, &$variant);

            buf.clear();
            let packed = PackedStruct(0x1020u16.to_be(), $variant);
            let r = buf.store(&packed);
            assert_eq!(&buf.as_slice()[0..2], &[16, 32]);
            assert_eq!(&buf.as_slice()[2..], slice);
            assert_eq!(
                unsafe { ptr::read_unaligned(ptr::addr_of!(buf.load(r)?.1)) },
                $variant
            );
        }};
    }

    let variant1 = Enum::Variant1;
    let variant2 = Enum::Variant2(0x02030405u32.to_be());
    let variant3 = Enum::Variant3(
        InnerStruct(0x02030405u32.to_be(), 6),
        0x060708090a0b0c0du64.to_be(),
    );

    test_case!(
        variant1,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    test_case!(
        variant2,
        &[1, 0, 0, 0, 2, 3, 4, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    test_case!(
        variant3,
        &[2, 0, 0, 0, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0, 0, 0, 6, 7, 8, 9, 10, 11, 12, 13]
    );
    Ok(())
}
