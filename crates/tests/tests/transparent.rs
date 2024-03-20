#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(transparent)]
struct TransparentStruct {
    string: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(transparent)]
struct TransparentTuple(String);

#[derive(Debug, PartialEq, Encode, Decode)]
enum TransparentEnum {
    NotTransparent {
        a: u32,
        b: u32,
    },
    #[musli(transparent)]
    Transparent(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct TransparentEnumUnpacked {
    type_tag: u8,
    variant_tag_type: u8,
    variant_tag: u8,
    value_type: u8,
    value: u32,
}

#[test]
fn transparent_struct() {
    tests::rt!(TransparentStruct {
        string: String::from("Hello"),
    });
    let string = tests::wire::transcode::<_, String>(TransparentStruct {
        string: String::from("Hello"),
    });
    assert_eq!(string, "Hello");

    tests::rt!(TransparentTuple(String::from("Hello")));
    let string = tests::wire::transcode::<_, String>(TransparentTuple(String::from("Hello")));
    assert_eq!(string, "Hello");
}

#[test]
fn transparent_enum() {
    tests::rt!(TransparentEnum::Transparent(42));
    tests::rt!(TransparentEnum::NotTransparent { a: 1, b: 2 });
}
