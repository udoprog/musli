#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Inner {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct AllSkipped {
    #[musli(skip)]
    skip: u32,
    #[musli(skip, default = skip_default)]
    skip_default: u32,
    #[musli(skip, default = skip_complex_field)]
    complex_field: Option<Inner>,
}

fn skip_default() -> u32 {
    42
}

fn skip_complex_field() -> Option<Inner> {
    Some(Inner { a: 1, b: 2 })
}

#[test]
fn skip() {
    tests::rt!(
        full,
        AllSkipped {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        },
        json = r#"{}"#,
    );

    tests::assert_decode_eq!(
        full,
        AllSkipped {
            skip: 10,
            skip_default: 52,
            complex_field: Some(Inner { a: 3, b: 4 }),
        },
        AllSkipped {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        },
        json = r#"{}"#,
    );
}
