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
    #[musli(skip = 42)]
    skip_default: u32,
    #[musli(skip = Some(Inner { a: 1, b: 2 }))]
    complex_field: Option<Inner>,
}

#[test]
fn skip() {
    tests::rt!(AllSkipped {
        skip: 0,
        skip_default: 42,
        complex_field: Some(Inner { a: 1, b: 2 }),
    });

    tests::assert_decode_eq!(
        AllSkipped {
            skip: 10,
            skip_default: 52,
            complex_field: Some(Inner { a: 3, b: 4 }),
        },
        AllSkipped {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        }
    );
}
