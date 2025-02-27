#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode)]
#[musli(untagged)]
pub enum OneWay {
    #[musli(packed)]
    EmptyVariant1,
    #[musli(packed)]
    EmptyVariant2,
    #[musli(packed)]
    StringVariant { value: String },
    #[musli(packed)]
    IntegerVariant { value: u32 },
    #[musli(packed)]
    StringTupleVariant(String, String),
    #[musli(packed)]
    IntegerTupleVariant(u32, u32),
}

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct StringVariant {
    value: String,
}

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct StringTupleVariant(String, String);

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct IntegerTupleVariant(u32, u32);

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct IntegerVariant {
    value: u32,
}

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct EmptyVariant;

/// Untagged enums may only implement `Encode`, and will be encoded according to
/// the exact specification of fields part of the variant.
#[test]
fn one_way_untagged_enum() {
    musli::macros::assert_decode_eq! {
        full,
        OneWay::EmptyVariant1,
        EmptyVariant,
        json = r#"[]"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        OneWay::EmptyVariant2,
        EmptyVariant,
        json = r#"[]"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        OneWay::StringVariant { value: String::from("Hello World") },
        StringVariant { value: String::from("Hello World") },
        json = r#"["Hello World"]"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        OneWay::IntegerVariant { value: 421 },
        IntegerVariant { value: 421 },
        json = r#"[421]"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        OneWay::StringTupleVariant(String::from("Hello..."), String::from("World!")),
        StringTupleVariant(String::from("Hello..."), String::from("World!")),
        json = r#"["Hello...","World!"]"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        OneWay::IntegerTupleVariant(10, 20),
        IntegerTupleVariant(10, 20),
        json = r#"[10,20]"#,
    };
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct {
    foo: u32,
    bar: u32,
    baz: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(untagged)]
pub enum Untagged {
    Person {
        name: String,
        age: u32,
    },
    #[musli(transparent)]
    Struct(Struct),
}

#[test]
fn untagged_enum() {
    musli::macros::assert_roundtrip_eq! {
        full,
        Untagged::Person {
            name: String::from("John"),
            age: 37,
        },
        json = r#"{"name":"John","age":37}"#,
    };

    musli::macros::assert_roundtrip_eq! {
        full,
        Untagged::Struct(Struct {
            foo: 1,
            bar: 2,
            baz: 3,
        }),
        json = r#"{"foo":1,"bar":2,"baz":3}"#,
    };
}
