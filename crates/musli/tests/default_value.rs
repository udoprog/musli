#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[allow(dead_code)]
struct Struct<'a> {
    name: &'a str,
    age: u32,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructDefault<'a> {
    name: &'a str,
    #[musli(default, skip_encoding_if = is_zero)]
    age: u32,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithOption<'a> {
    name: &'a str,
    #[musli(default, skip_encoding_if = Option::is_none)]
    age: Option<u32>,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructDefaultValue<'a> {
    name: &'a str,
    #[musli(default = default_age, skip_encoding_if = is_zero)]
    age: u32,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructSkip<'a> {
    name: &'a str,
    #[musli(skip)]
    age: u32,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructSkipDefault<'a> {
    name: &'a str,
    #[musli(skip, default)]
    age: u32,
    country: &'a str,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructSkipValue<'a> {
    name: &'a str,
    #[musli(skip, default = default_age)]
    age: u32,
    country: &'a str,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn default_age() -> u32 {
    180
}

// Ensure that skipped over fields ensures compatibility.
#[test]
fn decode_with_default() -> Result<(), Box<dyn std::error::Error>> {
    static NAME: &str = "Aristotle";
    static COUNTRY: &str = "Greece";

    musli::macros::assert_decode_eq!(
        full,
        StructDefault {
            name: NAME,
            age: 0,
            country: COUNTRY
        },
        StructWithOption {
            name: NAME,
            age: None,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructWithOption {
            name: NAME,
            age: None,
            country: COUNTRY
        },
        StructWithOption {
            name: NAME,
            age: None,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructWithOption {
            name: NAME,
            age: None,
            country: COUNTRY
        },
        StructDefault {
            name: NAME,
            age: 0,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructDefaultValue {
            name: NAME,
            age: 0,
            country: COUNTRY
        },
        StructDefaultValue {
            name: NAME,
            age: 180,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructDefaultValue {
            name: NAME,
            age: 170,
            country: COUNTRY
        },
        StructDefaultValue {
            name: NAME,
            age: 170,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"age":170,"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructSkip {
            name: NAME,
            age: 170,
            country: COUNTRY
        },
        StructDefault {
            name: NAME,
            age: 0,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructSkipDefault {
            name: NAME,
            age: 170,
            country: COUNTRY
        },
        StructDefault {
            name: NAME,
            age: 0,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    musli::macros::assert_decode_eq!(
        full,
        StructSkip {
            name: NAME,
            age: 170,
            country: COUNTRY
        },
        StructSkipValue {
            name: NAME,
            age: 180,
            country: COUNTRY
        },
        json = format!(r#"{{"name":{NAME:?},"country":{COUNTRY:?}}}"#),
    );

    Ok(())
}
