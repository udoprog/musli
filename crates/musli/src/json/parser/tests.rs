#![cfg(feature = "std")]

use rust_alloc::format;

use crate::context;
use crate::json::error::Error;
use crate::json::parser::integer::{parse_signed_full, parse_unsigned_full};
use crate::json::parser::SliceParser;

#[test]
fn test_decode_exponent() {
    crate::alloc::default(|alloc| {
        let cx = context::Same::<Error<_>, _>::new_in(alloc);

        macro_rules! test_number {
            ($ty:ty, $num:expr, $expected:expr) => {
                assert_eq!(
                    parse_unsigned_full::<$ty, _, _>(&cx, &mut SliceParser::new($num.as_bytes()))
                        .unwrap(),
                    $expected
                );
            };
        }

        macro_rules! test {
            ($expr:expr, $expected:expr) => {
                test_number!(u64, $expr, $expected);
                test_number!(u128, $expr, $expected);
                test_number!(usize, $expr, $expected);
            };
        }

        test!("0.01e4", 100);
        test!("1.01e4", 10100);
        test!("1.0100e4", 10100);
        test!("1.010000000e4", 10100);
        test!("1.01e8", 101000000);
        test!("1.0100001e8", 101000010);
        test!("1.0100001e7", 10100001);
        test!("1.321e3", 1321);
        test!("0.321e3", 321);
        test!("4000e-3", 4);
        test!("40000e-3", 40);
    })
}

#[test]
fn test_decode_unsigned() {
    crate::alloc::default(|alloc| {
        let cx = context::Same::<Error<_>, _>::new_in(alloc);

        macro_rules! test_number {
            ($ty:ty, $num:expr) => {
                assert_eq!(
                    parse_unsigned_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_unsigned_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_unsigned_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.0", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_unsigned_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.00000", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert!(parse_unsigned_full::<$ty, _, _>(
                    &cx,
                    &mut SliceParser::new(format!("{}.1", $num).as_bytes())
                )
                .is_err());
            };
        }

        macro_rules! test {
            ($ty:ty) => {
                test_number!($ty, 0);
                test_number!($ty, <$ty>::MIN);
                test_number!($ty, <$ty>::MAX);
            };
        }

        test!(u8);
        test!(u16);
        test!(u32);
        test!(u64);
        test!(u128);
        test!(usize);
    })
}

#[test]
fn test_decode_signed() {
    crate::alloc::default(|alloc| {
        let cx = context::Same::<Error<_>, _>::new_in(alloc);

        macro_rules! test_number {
            ($ty:ty, $num:expr) => {
                assert_eq!(
                    parse_signed_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_signed_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_signed_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.0", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert_eq!(
                    parse_signed_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.00000", $num).as_bytes())
                    )
                    .unwrap(),
                    $num
                );

                assert!(parse_signed_full::<$ty, _, _>(
                    &cx,
                    &mut SliceParser::new(format!("{}.1", $num).as_bytes())
                )
                .is_err());
            };
        }

        macro_rules! test {
            ($ty:ty) => {
                test_number!($ty, 0);
                test_number!($ty, -1);
                test_number!($ty, <$ty>::MIN);
                test_number!($ty, <$ty>::MAX);
            };
        }

        test!(i8);
        test!(i16);
        test!(i32);
        test!(i64);
        test!(i128);
        test!(isize);
    })
}
