#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SkipSerializeInner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SkipSerializeOuter {
    pub flag: bool,
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub inner: Option<SkipSerializeInner>,
}

#[test]
fn skip_serializing_if_outer() {
    musli::macros::assert_roundtrip_eq!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: Some(SkipSerializeInner),
        },
        json = r#"{"flag":false,"inner":{}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: None,
        },
        json = r#"{"flag":false}"#,
    );
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum External {
    Variant {
        name: String,
        #[musli(default, skip_encoding_if = is_zero)]
        a: u32,
        #[musli(default, skip_encoding_if = is_zero)]
        b: u32,
    },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type")]
enum Internal {
    #[musli(name_all = "name")]
    Variant {
        name: String,
        #[musli(default, skip_encoding_if = is_zero)]
        a: u32,
        #[musli(default, skip_encoding_if = is_zero)]
        b: u32,
    },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type", content = "content")]
enum Adjacent {
    Variant {
        name: String,
        #[musli(default, skip_encoding_if = is_zero)]
        a: u32,
        #[musli(default, skip_encoding_if = is_zero)]
        b: u32,
    },
}

// The map length of a variant body with conditionally skipped fields used to
// be taken from the enum's size hint, which for externally tagged enums is the
// size of the variant map, so formats which encode the length up front
// produced a corrupt map.
#[test]
fn skip_encoding_if_external_variant() {
    for (a, b) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        musli::macros::assert_roundtrip_eq!(
            full,
            External::Variant {
                name: String::from("Aristotle"),
                a,
                b,
            }
        );
    }
}

#[test]
fn skip_encoding_if_internal_variant() {
    for (a, b) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        musli::macros::assert_roundtrip_eq!(
            descriptive,
            Internal::Variant {
                name: String::from("Aristotle"),
                a,
                b,
            }
        );
    }
}

#[test]
fn skip_encoding_if_adjacent_variant() {
    for (a, b) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        musli::macros::assert_roundtrip_eq!(
            full,
            Adjacent::Variant {
                name: String::from("Aristotle"),
                a,
                b,
            }
        );
    }
}
