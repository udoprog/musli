use crate::reader::integer::{decode_signed, decode_unsigned};
use crate::reader::SliceParser;

#[test]
fn test_decode_exponent() {
    macro_rules! test_number {
        ($ty:ty, $num:expr, $expected:expr) => {
            assert_eq!(
                decode_unsigned::<$ty, _>(&mut SliceParser::new($num.as_bytes())).unwrap(),
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
}

#[test]
fn test_decode_unsigned() {
    macro_rules! test_number {
        ($ty:ty, $num:expr) => {
            assert_eq!(
                decode_unsigned::<$ty, _>(&mut SliceParser::new(format!("{}", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_unsigned::<$ty, _>(&mut SliceParser::new(format!("{}.", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_unsigned::<$ty, _>(&mut SliceParser::new(format!("{}.0", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_unsigned::<$ty, _>(&mut SliceParser::new(
                    format!("{}.00000", $num).as_bytes()
                ))
                .unwrap(),
                $num
            );

            assert!(decode_unsigned::<$ty, _>(&mut SliceParser::new(
                format!("{}.1", $num).as_bytes()
            ))
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
}

#[test]
fn test_decode_signed() {
    macro_rules! test_number {
        ($ty:ty, $num:expr) => {
            assert_eq!(
                decode_signed::<$ty, _>(&mut SliceParser::new(format!("{}", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_signed::<$ty, _>(&mut SliceParser::new(format!("{}.", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_signed::<$ty, _>(&mut SliceParser::new(format!("{}.0", $num).as_bytes()))
                    .unwrap(),
                $num
            );

            assert_eq!(
                decode_signed::<$ty, _>(&mut SliceParser::new(
                    format!("{}.00000", $num).as_bytes()
                ))
                .unwrap(),
                $num
            );

            assert!(decode_signed::<$ty, _>(&mut SliceParser::new(
                format!("{}.1", $num).as_bytes()
            ))
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
}
