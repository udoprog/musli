#![cfg(all(feature = "std", feature = "alloc"))]

use std::collections::BTreeMap;

use musli::json::{self, Encoding, Pretty};
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Address {
    street: String,
    city: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Person {
    name: String,
    age: u32,
    tags: Vec<String>,
    address: Address,
    empty_list: Vec<u32>,
    empty_map: BTreeMap<String, u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Kind {
    Struct {
        a: u32,
        b: u32,
    },
    #[musli(packed)]
    Tuple(u32, u32),
    Unit,
}

fn person() -> Person {
    Person {
        name: String::from("Aristotle"),
        age: 61,
        tags: vec![String::from("philosopher"), String::from("greek")],
        address: Address {
            street: String::from("Main St"),
            city: String::from("Athens"),
        },
        empty_list: Vec::new(),
        empty_map: BTreeMap::new(),
    }
}

const EXPECTED: &str = r#"{
  "name": "Aristotle",
  "age": 61,
  "tags": [
    "philosopher",
    "greek"
  ],
  "address": {
    "street": "Main St",
    "city": "Athens"
  },
  "empty_list": [],
  "empty_map": {}
}"#;

#[test]
fn to_string_pretty() {
    let person = person();
    assert_eq!(json::to_string_pretty(&person).unwrap(), EXPECTED);

    let actual: Person = json::from_str(EXPECTED).unwrap();
    assert_eq!(actual, person);
}

#[test]
fn to_vec_pretty() {
    assert_eq!(json::to_vec_pretty(&person()).unwrap(), EXPECTED.as_bytes());
}

#[test]
fn to_writer_pretty() {
    let mut data = Vec::new();
    json::to_writer_pretty(&mut data, &person()).unwrap();
    assert_eq!(data, EXPECTED.as_bytes());
}

#[test]
fn to_slice_pretty() {
    let mut buf = [0u8; 256];
    let w = json::to_slice_pretty(&mut buf[..], &person()).unwrap();
    assert_eq!(&buf[..w], EXPECTED.as_bytes());
}

#[test]
fn encoding_stays_compact_by_default() {
    assert_eq!(
        json::to_string(&person()).unwrap(),
        r#"{"name":"Aristotle","age":61,"tags":["philosopher","greek"],"address":{"street":"Main St","city":"Athens"},"empty_list":[],"empty_map":{}}"#
    );
}

#[test]
fn custom_indent() {
    const ENCODING: Encoding = Encoding::new().with_pretty(Pretty::new().with_indent(4));

    assert_eq!(
        ENCODING.to_string(&vec![vec![1u32], vec![2]]).unwrap(),
        "[\n    [\n        1\n    ],\n    [\n        2\n    ]\n]"
    );

    // An indentation of zero is the same as compact output.
    const COMPACT: Encoding = Encoding::new().with_compact();
    assert_eq!(
        COMPACT.to_string(&vec![vec![1u32], vec![2]]).unwrap(),
        "[[1],[2]]"
    );
}

#[test]
fn variants() {
    const ENCODING: Encoding = Encoding::new().with_pretty(Pretty::new());

    assert_eq!(
        ENCODING.to_string(&Kind::Struct { a: 1, b: 2 }).unwrap(),
        "{\n  \"Struct\": {\n    \"a\": 1,\n    \"b\": 2\n  }\n}"
    );

    assert_eq!(
        ENCODING.to_string(&Kind::Tuple(7, 8)).unwrap(),
        "{\n  \"Tuple\": [\n    7,\n    8\n  ]\n}"
    );

    assert_eq!(json::to_string(&Kind::Unit).unwrap(), r#"{"Unit":{}}"#);

    assert_eq!(
        ENCODING.to_string(&Kind::Unit).unwrap(),
        "{\n  \"Unit\": {}\n}"
    );

    for kind in [Kind::Struct { a: 1, b: 2 }, Kind::Tuple(7, 8), Kind::Unit] {
        let string = ENCODING.to_string(&kind).unwrap();
        let actual: Kind = json::from_str(&string).unwrap();
        assert_eq!(actual, kind);
    }
}

#[test]
fn scalars_are_untouched() {
    assert_eq!(json::to_string_pretty(&42u32).unwrap(), "42");
    assert_eq!(json::to_string_pretty(&"hello").unwrap(), "\"hello\"");
    assert_eq!(
        json::to_string_pretty(&Option::<u32>::None).unwrap(),
        "null"
    );
    assert_eq!(json::to_string_pretty(&Vec::<u32>::new()).unwrap(), "[]");
    assert_eq!(
        json::to_string_pretty(&BTreeMap::<String, u32>::new()).unwrap(),
        "{}"
    );
}

#[test]
fn byte_arrays_are_indented() {
    #[derive(Encode)]
    struct Bytes<'a> {
        #[musli(bytes)]
        data: &'a [u8],
    }

    assert_eq!(
        json::to_string_pretty(&Bytes { data: &[1, 2, 3] }).unwrap(),
        "{\n  \"data\": [\n    1,\n    2,\n    3\n  ]\n}"
    );

    assert_eq!(
        json::to_string_pretty(&Bytes { data: &[] }).unwrap(),
        "{\n  \"data\": []\n}"
    );
}

/// Encoding into a caller supplied writer is pretty printed just the same as
/// the `to_*` helpers are.
#[test]
fn encode_into_writer() {
    const ENCODING: Encoding = Encoding::new().with_pretty(Pretty::new());

    let mut buf = Vec::new();
    ENCODING.encode(&mut buf, &person()).unwrap();

    assert_eq!(buf, EXPECTED.as_bytes());
}
