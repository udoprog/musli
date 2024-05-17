#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct EmptyStruct;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct2(String);

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct3(String, u32);

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct4 {
    value: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct5 {
    value: String,
    value2: u32,
}

#[test]
fn structs() {
    musli::macros::assert_roundtrip_eq!(full, EmptyStruct);
    musli::macros::assert_roundtrip_eq!(full, Struct2(String::from("Hello World")));
    musli::macros::assert_roundtrip_eq!(full, Struct3(String::from("Hello World"), 42));
    musli::macros::assert_roundtrip_eq!(
        full,
        Struct4 {
            value: String::from("Hello World"),
        }
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        Struct5 {
            value: String::from("Hello World"),
            value2: 42,
        }
    );
}
