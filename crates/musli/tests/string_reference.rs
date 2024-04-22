#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithStr<'a> {
    name: &'a str,
    age: u32,
}

#[test]
fn string_reference() {
    let data = musli::wire::to_vec(&StructWithStr {
        name: "Jane Doe",
        age: 42,
    })
    .unwrap();

    let with_str: StructWithStr<'_> = musli::wire::decode(data.as_slice()).unwrap();
    assert_eq!(with_str.name, "Jane Doe");
    assert_eq!(with_str.age, 42);
}
