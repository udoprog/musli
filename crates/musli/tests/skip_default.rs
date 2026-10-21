#![cfg(feature = "test")]

use std::collections::BTreeMap;

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Plain {
    name: String,
    #[musli(skip_default)]
    age: u32,
    #[musli(skip_default)]
    flag: bool,
    #[musli(skip_default)]
    nick: Option<String>,
    #[musli(skip_default)]
    numbers: Vec<u32>,
    #[musli(skip_default)]
    map: BTreeMap<String, u32>,
    #[musli(skip_default)]
    text: String,
}

impl Plain {
    fn empty() -> Self {
        Self {
            name: String::from("Aristotle"),
            age: 0,
            flag: false,
            nick: None,
            numbers: Vec::new(),
            map: BTreeMap::new(),
            text: String::new(),
        }
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct OnlyName {
    name: String,
}

fn default_columns() -> u32 {
    3
}

fn shown() -> bool {
    true
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Custom {
    name: String,
    #[musli(skip_default, default = default_columns)]
    columns: u32,
    #[musli(default = shown, skip_default)]
    shown: bool,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Explicit {
    name: String,
    // `default` alongside `skip_default` is redundant but allowed.
    #[musli(default, skip_default)]
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Enum {
    Variant {
        name: String,
        #[musli(skip_default)]
        age: u32,
        #[musli(skip_default, default = default_columns)]
        columns: u32,
    },
    Tuple(String, #[musli(skip_default)] u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct ModeScoped {
    name: String,
    #[musli(Text, skip_default)]
    #[musli(Binary, default)]
    age: u32,
}

#[test]
fn plain_skipped_when_default() {
    musli::macros::assert_roundtrip_eq!(full, Plain::empty(), json = r#"{"name":"Aristotle"}"#);
}

#[test]
fn plain_encoded_when_not_default() {
    let mut map = BTreeMap::new();
    map.insert(String::from("a"), 1);

    musli::macros::assert_roundtrip_eq!(
        full,
        Plain {
            name: String::from("Aristotle"),
            age: 61,
            flag: true,
            nick: Some(String::new()),
            numbers: vec![1, 2],
            map: map.clone(),
            text: String::from("hi"),
        },
        json = r#"{"name":"Aristotle","age":61,"flag":true,"nick":"","numbers":[1,2],"map":{"a":1},"text":"hi"}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        Plain {
            age: 61,
            ..Plain::empty()
        },
        json = r#"{"name":"Aristotle","age":61}"#,
    );
}

#[test]
fn plain_missing_fields_decode_as_default() {
    musli::macros::assert_decode_eq!(
        full,
        OnlyName {
            name: String::from("Aristotle"),
        },
        Plain::empty(),
        json = r#"{"name":"Aristotle"}"#,
    );
}

#[test]
fn custom_skipped_when_equal_to_function() {
    musli::macros::assert_roundtrip_eq!(
        full,
        Custom {
            name: String::from("Aristotle"),
            columns: 3,
            shown: true,
        },
        json = r#"{"name":"Aristotle"}"#,
    );
}

#[test]
fn custom_encoded_when_equal_to_type_default() {
    // `Default::default()` differs from the `default = <path>` value, so it
    // must be encoded to roundtrip.
    musli::macros::assert_roundtrip_eq!(
        full,
        Custom {
            name: String::from("Aristotle"),
            columns: 0,
            shown: false,
        },
        json = r#"{"name":"Aristotle","columns":0,"shown":false}"#,
    );
}

#[test]
fn custom_missing_fields_decode_from_function() {
    musli::macros::assert_decode_eq!(
        full,
        OnlyName {
            name: String::from("Aristotle"),
        },
        Custom {
            name: String::from("Aristotle"),
            columns: 3,
            shown: true,
        },
        json = r#"{"name":"Aristotle"}"#,
    );
}

#[test]
fn explicit_default() {
    musli::macros::assert_roundtrip_eq!(
        full,
        Explicit {
            name: String::from("Aristotle"),
            age: 0,
        },
        json = r#"{"name":"Aristotle"}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        Explicit {
            name: String::from("Aristotle"),
            age: 1,
        },
        json = r#"{"name":"Aristotle","age":1}"#,
    );
}

#[test]
fn enum_variants() {
    musli::macros::assert_roundtrip_eq!(
        full,
        Enum::Variant {
            name: String::from("Aristotle"),
            age: 0,
            columns: 3,
        },
        json = r#"{"Variant":{"name":"Aristotle"}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        Enum::Variant {
            name: String::from("Aristotle"),
            age: 61,
            columns: 0,
        },
        json = r#"{"Variant":{"name":"Aristotle","age":61,"columns":0}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        Enum::Tuple(String::from("Aristotle"), 0),
        json = r#"{"Tuple":{"0":"Aristotle"}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        Enum::Tuple(String::from("Aristotle"), 61),
        json = r#"{"Tuple":{"0":"Aristotle","1":61}}"#,
    );
}

#[test]
fn mode_scoped() {
    fn value() -> ModeScoped {
        ModeScoped {
            name: String::from("Aristotle"),
            age: 0,
        }
    }

    musli::macros::assert_roundtrip_eq!(full, value(), json = r#"{"name":"Aristotle"}"#);

    // In Binary mode the field is always encoded.
    musli::macros::assert_decode_eq!(
        full,
        value(),
        value(),
        json_binary = r#"{"0":"Aristotle","1":0}"#
    );
}
