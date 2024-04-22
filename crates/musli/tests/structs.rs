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
    musli::rt!(full, EmptyStruct);
    musli::rt!(full, Struct2(String::from("Hello World")));
    musli::rt!(full, Struct3(String::from("Hello World"), 42));
    musli::rt!(
        full,
        Struct4 {
            value: String::from("Hello World"),
        }
    );
    musli::rt!(
        full,
        Struct5 {
            value: String::from("Hello World"),
            value2: 42,
        }
    );
}
