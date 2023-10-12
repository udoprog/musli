#![allow(clippy::assertions_on_constants)]

use core::marker::PhantomData;
use core::mem::size_of;

use crate::pointer::Ref;
use crate::{AlignedBuf, Error, ZeroCopy};

#[test]
fn test_weird_alignment() -> Result<(), Error> {
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

    let mut buf = AlignedBuf::with_alignment::<WeirdAlignment>();
    let w = buf.store(&weird)?;
    let buf = buf.as_aligned();

    assert_eq!(buf.len(), size_of::<WeirdAlignment>());
    assert_eq!(buf.load(w)?, &weird);
    Ok(())
}

#[test]
fn test_enum_boundaries() -> Result<(), Error> {
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

            let mut buf = AlignedBuf::with_alignment::<$name>();
            let v1 = buf.store(&$name::Variant1)?;
            let v2 = buf.store(&$name::Variant2)?;
            let v3 = buf.store(&$name::Variant3)?;
            let max = buf.store(&$name::Max)?;
            let min = buf.store(&$name::Min)?;
            let after_min = buf.store(&$name::AfterMin)?;
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX - 1))?.offset());

            let buf = buf.as_aligned();

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
fn test_signed_wraparound() -> Result<(), Error> {
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

            let mut buf = AlignedBuf::with_alignment::<$name>();
            let minus_one = buf.store(&$name::MinusOne)?;
            let zero = buf.store(&$name::Zero)?;
            let one = buf.store(&$name::One)?;
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX))?.offset());

            let buf = buf.as_aligned();

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
fn test_neg0() -> Result<(), Error> {
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

            let mut buf = AlignedBuf::with_alignment::<$name>();
            let minus_one = buf.store(&$name::MinusOne)?;
            let neg0 = buf.store(&$name::Neg0)?;
            let one = buf.store(&$name::One)?;
            let v4 = Ref::<$name>::new(buf.store(&(<$num>::MAX))?.offset());

            let buf = buf.as_aligned();

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
fn test_needs_padding() -> Result<(), Error> {
    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct Zst {}

    const _: () = assert!(!Zst::NEEDS_PADDING);

    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct SingleField {
        not_padded: u32,
    }

    const _: () = assert!(!SingleField::NEEDS_PADDING);

    #[derive(ZeroCopy)]
    #[repr(transparent)]
    #[zero_copy(crate)]
    struct MightPad {
        might_pad: [u32; 4],
    }

    const _: () = assert!(!MightPad::NEEDS_PADDING);
    Ok(())
}

#[test]
fn test_enum_with_fields() -> Result<(), Error> {
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

    let mut buf = AlignedBuf::new();
    let variant = buf.store(&Types::Variant {
        field: 10,
        field2: 20,
    })?;
    let variant2 = buf.store(&Types::Variant2(40))?;
    let variant3 = buf.store(&Types::Variant3)?;
    let empty = buf.store(&Types::Empty { empty: PhantomData })?;

    let buf = buf.as_aligned();

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
