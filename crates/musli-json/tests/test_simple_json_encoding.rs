#![cfg(feature = "std")]

use musli::{Decode, Encode};
use musli_json::Encoding;
use rand::prelude::*;

// M marker indicating that some attributes should only apply when we're
// decoding in a JSON mode.
mod my_modes {
    use musli::mode::Mode;

    pub(crate) enum Json {}

    impl Mode for Json {}
}

const CONFIG: Encoding<my_modes::Json> = Encoding::new().with_mode();

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(mode = my_modes::Json, default_field_name = "name")]
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

    let out = musli_json::to_string(&expected).unwrap();
    println!("{}", out);
    let actual: Vec<SimpleJsonStruct<'_>> = musli_json::from_slice(out.as_bytes()).unwrap();
    assert_eq!(expected, actual);

    let value: musli_value::Value = musli_json::from_slice(b"10.00001e-121").unwrap();
    println!("{:?}", value);
    // assert_eq!(expected, actual);
}
