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
#[allow(dead_code)]
struct TransparentEnumUnpacked {
    type_tag: u8,
    variant_tag_type: u8,
    variant_tag: u8,
    value_type: u8,
    value: u32,
}

#[test]
fn transparent_struct() {
    musli::macros::assert_roundtrip_eq!(
        full,
        TransparentStruct {
            string: String::from("Hello World"),
        }
    );

    musli::macros::assert_roundtrip_eq!(full, TransparentTuple(String::from("Hello World")));

    musli::macros::assert_decode_eq!(
        full,
        TransparentStruct {
            string: String::from("Hello World"),
        },
        String::from("Hello World"),
        json = r#""Hello World""#,
    );

    musli::macros::assert_decode_eq!(
        full,
        TransparentTuple(String::from("Hello World")),
        String::from("Hello World"),
        json = r#""Hello World""#,
    );
}

#[test]
fn transparent_enum() {
    musli::macros::assert_roundtrip_eq!(full, TransparentEnum::Transparent(42));
    musli::macros::assert_roundtrip_eq!(full, TransparentEnum::NotTransparent { a: 1, b: 2 });
}
