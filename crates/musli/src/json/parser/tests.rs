#![cfg(feature = "std")]

use rust_alloc::format;

use crate::context;
use crate::json::parser::SliceParser;
use crate::json::parser::integer::{parse_signed_full, parse_unsigned_full};

#[test]
fn test_decode_exponent() {
    crate::alloc::default(|alloc| {
        let cx = context::new_in(alloc);

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
        let cx = context::new_in(alloc);

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

                assert!(
                    parse_unsigned_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.1", $num).as_bytes())
                    )
                    .is_err()
                );
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
        let cx = context::new_in(alloc);

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

                assert!(
                    parse_signed_full::<$ty, _, _>(
                        &cx,
                        &mut SliceParser::new(format!("{}.1", $num).as_bytes())
                    )
                    .is_err()
                );
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

/// Numbers which overflow the target type in the *last* digit must be reported
/// as an error rather than wrapping around.
#[test]
fn test_decode_overflow() {
    crate::alloc::default(|alloc| {
        let cx = context::new_in(alloc);

        macro_rules! test {
            ($parse:ident, $ty:ty, $num:expr) => {
                assert!(
                    $parse::<$ty, _, _>(&cx, &mut SliceParser::new($num.as_bytes())).is_err(),
                    "{} should not parse as {}",
                    $num,
                    stringify!($ty)
                );
            };
        }

        test!(parse_unsigned_full, u8, "256");
        test!(parse_unsigned_full, u16, "65536");
        test!(parse_unsigned_full, u32, "4294967299");
        test!(parse_unsigned_full, u64, "18446744073709551616");
        test!(parse_unsigned_full, u128, "340282366920938463463374607431768211456");
        test!(parse_signed_full, i8, "128");
        test!(parse_signed_full, i16, "32768");
        test!(parse_signed_full, i32, "2147483648");
        test!(parse_signed_full, i64, "9223372036854775808");

        // The exponent is accumulated with the same routine.
        test!(parse_unsigned_full, u64, "1e4294967299");
    })
}
