#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructFrom {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructTo {
    #[musli(mode = Binary, name = 1)]
    #[musli(mode = Text, name = "number")]
    string: u32,
    #[musli(mode = Binary, name = 0)]
    #[musli(mode = Text, name = "string")]
    number: String,
}

#[test]
fn struct_renamed() {
    let from = musli::macros::assert_roundtrip_eq!(
        full,
        StructFrom {
            string: String::from("a string"),
            number: 42,
        }
    );

    let to = musli::macros::assert_roundtrip_eq!(
        full,
        StructTo {
            string: 42,
            number: String::from("a string"),
        }
    );

    let out = musli::wire::to_vec(&from).expect("failed to encode");
    let value: StructTo = musli::wire::decode(out.as_slice()).expect("failed to decode");
    assert_eq!(value, to);
}
