#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructFrom {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructTo {
    #[musli(name = 1)]
    number: u32,
    #[musli(name = 0)]
    string: String,
}

#[test]
fn struct_renamed() {
    let from = tests::rt!(
        full,
        StructFrom {
            string: String::from("a string"),
            number: 42,
        }
    );

    let to = tests::rt!(
        full,
        StructTo {
            number: 42,
            string: String::from("a string"),
        }
    );

    let out = tests::wire::to_vec(&from).expect("failed to encode");
    let value: StructTo = tests::wire::decode(out.as_slice()).expect("failed to decode");
    assert_eq!(value, to);
}
