#![cfg(feature = "std")]

use musli::json::Encoding;
use musli::{Decode, Encode};
use rand::prelude::*;

pub(crate) enum Json {}

const CONFIG: Encoding<Json> = Encoding::new().with_mode();

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(mode = Json, name_all = "name")]
struct SimpleJsonStruct<'a> {
    name: &'a str,
    age: f32,
    c: char,
}

#[test]
fn test_simple_json_encoding() {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let expected = vec![SimpleJsonStruct {
        name: "Aristotle",
        age: 61.1415,
        c: rng.gen(),
    }];

    let out = CONFIG.to_string(&expected).unwrap();
    println!("{}", out);
    let actual: Vec<SimpleJsonStruct<'_>> = CONFIG.from_slice(out.as_bytes()).unwrap();
    assert_eq!(expected, actual);

    let out = musli::json::to_string(&expected).unwrap();
    println!("{}", out);
    let actual: Vec<SimpleJsonStruct<'_>> = musli::json::from_slice(out.as_bytes()).unwrap();
    assert_eq!(expected, actual);

    let value: musli::value::Value<_> = musli::json::from_slice(b"10.00001e-121").unwrap();
    println!("{:?}", value);
    // assert_eq!(expected, actual);
}
