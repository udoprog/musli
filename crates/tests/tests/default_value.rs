#![cfg(feature = "test")]

use musli::{Decode, Encode};

/// Empty enums should work.
#[derive(Encode, Decode)]
struct Struct {
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithDefault {
    name: String,
    #[musli(default, skip_encoding_if = is_zero)]
    age: u32,
    country: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithOption {
    name: String,
    #[musli(default, skip_encoding_if = Option::is_none)]
    age: Option<u32>,
    country: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithDefaultValue {
    name: String,
    #[musli(default = default_age, skip_encoding_if = is_zero)]
    age: u32,
    country: String,
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
    tests::assert_decode_eq!(
        full,
        StructWithDefault {
            name: String::from("Aristotle"),
            age: 0,
            country: String::from("Greece"),
        },
        StructWithOption {
            name: String::from("Aristotle"),
            age: None,
            country: String::from("Greece"),
        },
        json = r#"{"0":"Aristotle","2":"Greece"}"#,
    );

    tests::assert_decode_eq!(
        full,
        StructWithOption {
            name: String::from("Aristotle"),
            age: None,
            country: String::from("Greece"),
        },
        StructWithOption {
            name: String::from("Aristotle"),
            age: None,
            country: String::from("Greece"),
        },
        json = r#"{"0":"Aristotle","2":"Greece"}"#,
    );

    tests::assert_decode_eq!(
        full,
        StructWithOption {
            name: String::from("Aristotle"),
            age: None,
            country: String::from("Greece"),
        },
        StructWithDefault {
            name: String::from("Aristotle"),
            age: 0,
            country: String::from("Greece"),
        },
        json = r#"{"0":"Aristotle","2":"Greece"}"#,
    );

    tests::assert_decode_eq!(
        full,
        StructWithDefaultValue {
            name: String::from("Aristotle"),
            age: 0,
            country: String::from("Greece"),
        },
        StructWithDefaultValue {
            name: String::from("Aristotle"),
            age: 180,
            country: String::from("Greece"),
        },
        json = r#"{"0":"Aristotle","2":"Greece"}"#,
    );

    tests::assert_decode_eq!(
        full,
        StructWithDefaultValue {
            name: String::from("Aristotle"),
            age: 170,
            country: String::from("Greece"),
        },
        StructWithDefaultValue {
            name: String::from("Aristotle"),
            age: 170,
            country: String::from("Greece"),
        },
        json = r#"{"0":"Aristotle","1":170,"2":"Greece"}"#,
    );

    Ok(())
}
