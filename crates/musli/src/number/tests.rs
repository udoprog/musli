use rust_alloc::format;
use rust_alloc::string::ToString;

use super::parse::digit;
use super::{
    Any, Json, Json5, parse_any, parse_float, parse_signed, parse_signed_base, parse_unsigned,
    parse_unsigned_base,
};

/// A number written with an exponent still denotes a whole number as long as
/// nothing is left behind the point once it has been scaled.
#[test]
fn decode_exponent() {
    macro_rules! test_number {
        ($ty:ty, $num:expr, $expected:expr) => {
            assert_eq!(
                parse_unsigned::<Json, $ty>($num.as_bytes()).unwrap(),
                ($expected, $num.len())
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
}

#[test]
fn decode_unsigned() {
    macro_rules! test_number {
        ($ty:ty, $num:expr) => {
            for suffix in ["", ".", ".0", ".00000"] {
                let string = format!("{}{suffix}", $num);

                assert_eq!(
                    parse_unsigned::<Json, $ty>(string.as_bytes()).unwrap(),
                    ($num, string.len()),
                    "{string}"
                );
            }

            assert!(parse_unsigned::<Json, $ty>(format!("{}.1", $num).as_bytes()).is_err());
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
fn decode_signed() {
    macro_rules! test_number {
        ($ty:ty, $num:expr) => {
            for suffix in ["", ".", ".0", ".00000"] {
                let string = format!("{}{suffix}", $num);

                assert_eq!(
                    parse_signed::<Json, $ty>(string.as_bytes()).unwrap(),
                    ($num, string.len()),
                    "{string}"
                );
            }

            assert!(parse_signed::<Json, $ty>(format!("{}.1", $num).as_bytes()).is_err());
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

/// Numbers which overflow the target type in the *last* digit must be reported
/// as an error rather than wrapping around.
#[test]
fn decode_overflow() {
    macro_rules! test {
        ($parse:ident, $ty:ty, $num:expr) => {
            assert!(
                $parse::<Json, $ty>($num.as_bytes()).is_err(),
                "{} should not parse as {}",
                $num,
                stringify!($ty)
            );
        };
    }

    test!(parse_unsigned, u8, "256");
    test!(parse_unsigned, u16, "65536");
    test!(parse_unsigned, u32, "4294967299");
    test!(parse_unsigned, u64, "18446744073709551616");
    test!(
        parse_unsigned,
        u128,
        "340282366920938463463374607431768211456"
    );
    test!(parse_signed, i8, "128");
    test!(parse_signed, i16, "32768");
    test!(parse_signed, i32, "2147483648");
    test!(parse_signed, i64, "9223372036854775808");

    // The exponent is accumulated with the same routine.
    test!(parse_unsigned, u64, "1e4294967299");

    // The same must hold for the base-only routines.
    test!(parse_unsigned_base, u8, "256");
    test!(parse_unsigned_base, u16, "65536");
    test!(parse_unsigned_base, u32, "4294967299");
    test!(parse_unsigned_base, u64, "18446744073709551616");
    test!(
        parse_unsigned_base,
        u128,
        "340282366920938463463374607431768211456"
    );
    test!(parse_signed_base, i8, "128");
    test!(parse_signed_base, i16, "32768");
    test!(parse_signed_base, i32, "2147483648");
    test!(parse_signed_base, i64, "9223372036854775808");

    // Numbers with far more digits than the target type can hold.
    test!(parse_unsigned_base, u8, "999999999999");
    test!(parse_unsigned, u8, "999999999999");
    test!(parse_unsigned_base, u32, "999999999999999999999999");
    test!(parse_unsigned, u32, "999999999999999999999999");
    test!(parse_signed_base, i64, "-99999999999999999999999999");
    test!(parse_signed, i64, "-99999999999999999999999999");
}

/// The base-only routines stop at the point, leaving the rest of the number for
/// whoever asked to read it.
#[test]
fn decode_base_stops_early() {
    assert_eq!(
        parse_unsigned_base::<Json, u32>(b"123.45e6").unwrap(),
        (123, 3)
    );
    assert_eq!(
        parse_signed_base::<Json, i32>(b"-123.45e6").unwrap(),
        (-123, 4)
    );
}

/// A number is measured even when it is followed by the rest of a document.
#[test]
fn decode_stops_at_the_end_of_the_number() {
    assert_eq!(parse_unsigned::<Json, u32>(b"123,456").unwrap(), (123, 3));
    assert_eq!(parse_unsigned::<Json, u32>(b"1.5e2]").unwrap(), (150, 5));
}

/// The forms JSON5 adds on top of RFC 8259, which SQLite stores in its `INT5`
/// and `FLOAT5` elements.
#[test]
fn decode_json5() {
    assert_eq!(parse_unsigned::<Json5, u32>(b"0x1f").unwrap(), (31, 4));
    assert_eq!(parse_unsigned::<Json5, u32>(b"0X1F").unwrap(), (31, 4));
    assert_eq!(parse_signed::<Json5, i32>(b"-0x1f").unwrap(), (-31, 5));
    assert_eq!(parse_signed::<Json5, i32>(b"+17").unwrap(), (17, 3));
    assert_eq!(parse_unsigned::<Json5, u32>(b"007").unwrap(), (7, 3));
    assert_eq!(parse_float::<Json5, f64>(b".5").unwrap(), (0.5, 2));
    assert_eq!(parse_float::<Json5, f64>(b"0x20").unwrap(), (32.0, 4));
    assert_eq!(parse_float::<Json5, f64>(b"-0x20").unwrap(), (-32.0, 5));
    assert_eq!(
        parse_float::<Json5, f64>(b"Infinity").unwrap(),
        (f64::INFINITY, 8)
    );
    assert!(parse_float::<Json5, f64>(b"NaN").unwrap().0.is_nan());

    // None of which canonical JSON accepts. A hexadecimal number is read as
    // the `0` it starts with, leaving the `x` for whoever asked to read it.
    assert_eq!(parse_unsigned::<Json, u32>(b"0x1f").unwrap(), (0, 1));
    assert!(parse_signed::<Json, i32>(b"+17").is_err());
    assert!(parse_unsigned::<Json, u32>(b"007").is_err());
    // Floats go through the decimal to float conversion, which accepts a
    // superset of every syntax here, so `Json` tolerates the JSON5 spellings
    // too. Nothing routes them to it, since a JSON document is tokenized before
    // a number is read out of it.
    assert_eq!(
        parse_float::<Json, f64>(b"Infinity").unwrap(),
        (f64::INFINITY, 8)
    );
}

/// A hexadecimal integer is bounded by what it is being decoded into, the same
/// way a decimal one is.
#[test]
fn decode_hex_overflow() {
    assert_eq!(parse_unsigned::<Json5, u8>(b"0xff").unwrap(), (255, 4));
    assert!(parse_unsigned::<Json5, u8>(b"0x100").is_err());
    assert!(parse_unsigned::<Json5, u32>(b"0x").is_err());
}

/// Whatever is wrong with a number, the diagnostic says what was expected and
/// which byte of the number is at fault.
#[test]
fn diagnostics() {
    macro_rules! test {
        ($parse:ident::<$syntax:ty, $ty:ty>($input:expr), $at:expr, $expected:expr) => {
            let error = $parse::<$syntax, $ty>($input).unwrap_err();
            assert_eq!(
                (error.to_string().as_str(), error.at()),
                ($expected, $at),
                "{}",
                stringify!($input)
            );
        };
    }

    test!(
        parse_unsigned::<Json, u32>(b"abc"),
        0,
        "Expected a digit, but found `a`"
    );
    test!(
        parse_unsigned::<Json, u32>(b""),
        0,
        "Expected a digit, but the number ended"
    );
    test!(
        parse_unsigned::<Json, u32>(b"-1"),
        0,
        "Expected a digit, but found `-`"
    );
    test!(
        parse_unsigned::<Json, u32>(b"007"),
        0,
        "A number must not have a redundant leading zero"
    );
    test!(
        parse_unsigned::<Json, u32>(b"1e"),
        2,
        "Expected a digit in the exponent, but the number ended"
    );
    test!(
        parse_unsigned::<Json, u32>(b"1e+x"),
        3,
        "Expected a digit in the exponent, but found `x`"
    );
    test!(
        parse_unsigned::<Json, u32>(b"1.5"),
        2,
        "Expected a whole number, but found a fraction"
    );
    // A value which does not fit is reported against the number as a whole,
    // since which digit tipped it over is not what the reader needs to know.
    test!(
        parse_unsigned::<Json, u8>(b"1234"),
        0,
        "Arithmetic overflow"
    );
    test!(
        parse_unsigned::<Json, u32>(b"1e99999999999"),
        0,
        "Exponent is out of range"
    );
    test!(
        parse_float::<Json, f64>(b"nope"),
        0,
        "Expected a digit, but found `n`"
    );
    // A point on its own is not a number, even where one is allowed to lead.
    test!(
        parse_unsigned::<Json5, u32>(b"."),
        1,
        "Expected a digit in the fraction, but the number ended"
    );
    test!(
        parse_unsigned::<Json5, u32>(b".x"),
        1,
        "Expected a digit in the fraction, but found `x`"
    );
}

/// A number read without a type in mind lands on the narrowest thing which
/// holds it, and falls back to a float when nothing does.
#[test]
fn decode_any() {
    assert!(matches!(
        parse_any::<Json>(b"42").unwrap(),
        (Any::Unsigned(42), 2)
    ));
    assert!(matches!(
        parse_any::<Json>(b"-42").unwrap(),
        (Any::Signed(-42), 3)
    ));
    assert!(matches!(
        parse_any::<Json>(b"1.5").unwrap(),
        (Any::Float(1.5), 3)
    ));
    assert!(matches!(
        parse_any::<Json>(b"1e40").unwrap(),
        (Any::Float(1e40), 4)
    ));
    assert!(matches!(
        parse_any::<Json>(b"-1e40").unwrap(),
        (Any::Float(-1e40), 5)
    ));
    // Larger than any integer, but a float still has it.
    assert!(matches!(
        parse_any::<Json>(b"340282366920938463463374607431768211456").unwrap(),
        (Any::Float(..), 39)
    ));
    assert!(matches!(
        parse_any::<Json5>(b"0x1f").unwrap(),
        (Any::Unsigned(31), 4)
    ));
}

/// Every byte a digit could be, since the translation works on the bits of a
/// byte rather than on the ranges it is written as and the two are only the
/// same if nothing outside those ranges slips through.
#[test]
fn decode_digit() {
    for b in 0..=u8::MAX {
        let decimal = match b {
            b'0'..=b'9' => Some(b - b'0'),
            _ => None,
        };

        let hex = match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        };

        assert_eq!(digit::<10>(b), decimal, "{:?} in base ten", b as char);
        assert_eq!(digit::<16>(b), hex, "{:?} in base sixteen", b as char);
    }
}
