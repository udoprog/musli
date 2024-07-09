//! Test which ensures that compatible types can be decoded.

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[allow(dead_code)]
struct SignedIntegers {
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: i128,
    f: isize,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[allow(dead_code)]
struct UnsignedIntegers {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: usize,
}

#[test]
fn signed_to_unsigned() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            struct UnsignedIntegers {
                a: $ty,
                b: $ty,
                c: $ty,
                d: $ty,
                e: $ty,
                f: $ty,
            }

            musli::macros::assert_decode_eq! {
                descriptive,
                SignedIntegers {
                    a: 2,
                    b: 3,
                    c: 4,
                    d: 5,
                    e: 6,
                    f: 7,
                },
                UnsignedIntegers {
                    a: 2,
                    b: 3,
                    c: 4,
                    d: 5,
                    e: 6,
                    f: 7,
                }
            };
        }};
    }

    test_case!(u8);
    test_case!(u16);
    test_case!(u32);
    test_case!(u64);
    test_case!(u128);
    test_case!(usize);
}

#[test]
fn unsigned_to_signed() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            struct UnsignedIntegers {
                a: $ty,
                b: $ty,
                c: $ty,
                d: $ty,
                e: $ty,
                f: $ty,
            }

            musli::macros::assert_decode_eq! {
                descriptive,
                UnsignedIntegers {
                    a: 2,
                    b: 3,
                    c: 4,
                    d: 5,
                    e: 6,
                    f: 7,
                },
                SignedIntegers {
                    a: 2,
                    b: 3,
                    c: 4,
                    d: 5,
                    e: 6,
                    f: 7,
                }
            };
        }};
    }

    test_case!(i8);
    test_case!(i16);
    test_case!(i32);
    test_case!(i64);
    test_case!(i128);
    test_case!(isize);
}
