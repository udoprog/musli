#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
pub enum Enum {
    EmptyVariant1,
    EmptyVariant2,
    StringVariant { value: String },
    IntegerVariant { value: u32 },
    StringTupleVariant(String, String),
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
fn untagged_enums() -> Result<(), Box<dyn std::error::Error>> {
    musli::assert_decode_eq! {
        full,
        Enum::EmptyVariant1,
        EmptyVariant,
        json = r#"[]"#,
    };

    musli::assert_decode_eq! {
        full,
        Enum::EmptyVariant2,
        EmptyVariant,
        json = r#"[]"#,
    };

    musli::assert_decode_eq! {
        full,
        Enum::StringVariant { value: String::from("Hello World") },
        StringVariant { value: String::from("Hello World") },
        json = r#"["Hello World"]"#,
    };

    musli::assert_decode_eq! {
        full,
        Enum::IntegerVariant { value: 421 },
        IntegerVariant { value: 421 },
        json = r#"[421]"#,
    };

    musli::assert_decode_eq! {
        full,
        Enum::StringTupleVariant(String::from("Hello..."), String::from("World!")),
        StringTupleVariant(String::from("Hello..."), String::from("World!")),
        json = r#"["Hello...","World!"]"#,
    };

    musli::assert_decode_eq! {
        full,
        Enum::IntegerTupleVariant(10, 20),
        IntegerTupleVariant(10, 20),
        json = r#"[10,20]"#,
    };

    Ok(())
}
