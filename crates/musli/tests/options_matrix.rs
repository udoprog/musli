//! Round trip numbers through every combination of [`Options`] that the
//! non-descriptive and descriptive binary formats support.

#![cfg(all(feature = "storage", feature = "wire", feature = "descriptive"))]

use musli::options::{self, Options};
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Numbers {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: i8,
    g: i16,
    h: i32,
    i: i64,
    j: i128,
    m: f32,
    n: f64,
}

fn numbers() -> [Numbers; 4] {
    [
        Numbers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            g: 0,
            h: 0,
            i: 0,
            j: 0,
            m: 0.0,
            n: 0.0,
        },
        Numbers {
            a: u8::MAX,
            b: u16::MAX,
            c: u32::MAX,
            d: u64::MAX,
            e: u128::MAX,
            f: i8::MAX,
            g: i16::MAX,
            h: i32::MAX,
            i: i64::MAX,
            j: i128::MAX,
            m: f32::MAX,
            n: f64::MAX,
        },
        Numbers {
            a: 1,
            b: 1,
            c: 1,
            d: 1,
            e: 1,
            f: i8::MIN,
            g: i16::MIN,
            h: i32::MIN,
            i: i64::MIN,
            j: i128::MIN,
            m: f32::MIN,
            n: f64::MIN,
        },
        Numbers {
            a: 127,
            b: 128,
            c: 16384,
            d: 1 << 63,
            e: 1 << 127,
            f: -1,
            g: -1,
            h: -1,
            i: -1,
            j: -1,
            m: -0.0,
            n: f64::EPSILON,
        },
    ]
}

/// Platform sized integers, kept within the narrowest configured pointer width
/// so that every row of the matrix can represent them.
#[derive(Debug, PartialEq, Encode, Decode)]
struct Sizes {
    a: usize,
    b: isize,
}

macro_rules! matrix {
    ($($name:ident => $opt:expr),* $(,)?) => {
        $(
            const $name: Options = $opt;
        )*

        #[test]
        fn round_trip_numbers() {
            $({
                const STORAGE: musli::storage::Encoding<$name> =
                    musli::storage::Encoding::new().with_options();
                const WIRE: musli::wire::Encoding<$name> =
                    musli::wire::Encoding::new().with_options();
                const DESCRIPTIVE: musli::descriptive::Encoding<$name> =
                    musli::descriptive::Encoding::new().with_options();

                let name = stringify!($name);

                for value in &numbers() {
                    let bytes = STORAGE.to_vec(value).expect(name);
                    let back: Numbers = STORAGE.from_slice(&bytes).expect(name);
                    assert_eq!(&back, value, "storage {name}");

                    let bytes = WIRE.to_vec(value).expect(name);
                    let back: Numbers = WIRE.from_slice(&bytes).expect(name);
                    assert_eq!(&back, value, "wire {name}");

                    let bytes = DESCRIPTIVE.to_vec(value).expect(name);
                    let back: Numbers = DESCRIPTIVE.from_slice(&bytes).expect(name);
                    assert_eq!(&back, value, "descriptive {name}");
                }

                for value in [
                    Sizes { a: 0, b: 0 },
                    Sizes { a: u32::MAX as usize, b: i32::MAX as isize },
                    Sizes { a: 1, b: i32::MIN as isize },
                    Sizes { a: 127, b: -1 },
                ] {
                    let bytes = STORAGE.to_vec(&value).expect(name);
                    let back: Sizes = STORAGE.from_slice(&bytes).expect(name);
                    assert_eq!(back, value, "storage sizes {name}");

                    let bytes = WIRE.to_vec(&value).expect(name);
                    let back: Sizes = WIRE.from_slice(&bytes).expect(name);
                    assert_eq!(back, value, "wire sizes {name}");

                    let bytes = DESCRIPTIVE.to_vec(&value).expect(name);
                    let back: Sizes = DESCRIPTIVE.from_slice(&bytes).expect(name);
                    assert_eq!(back, value, "descriptive sizes {name}");
                }
            })*
        }
    };
}

matrix! {
    DEFAULT => options::new().build(),
    FIXED => options::new().fixed().build(),
    VARIABLE => options::new().variable().build(),
    FIXED_BE => options::new().fixed().byte_order(options::ByteOrder::Big).build(),
    FIXED_LE => options::new().fixed().byte_order(options::ByteOrder::Little).build(),
    FIXED_NATIVE => options::new().fixed().native_byte_order().build(),
    VARIABLE_BE => options::new().variable().byte_order(options::ByteOrder::Big).build(),
    FIXED_PTR32 => options::new().fixed().pointer(options::Width::U32).build(),
    FIXED_PTR64 => options::new().fixed().pointer(options::Width::U64).build(),
    VAR_PTR32 => options::new().variable().pointer(options::Width::U32).build(),
    INT_FIXED_FLOAT_VAR => options::new()
        .integer(options::Integer::Fixed)
        .float(options::Float::Variable)
        .build(),
    INT_VAR_FLOAT_FIXED => options::new()
        .integer(options::Integer::Variable)
        .float(options::Float::Fixed)
        .build(),
}
