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
    #[musli(default)]
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithOption {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

#[test]
fn decode_with_default() -> Result<(), Box<dyn std::error::Error>> {
    let name = String::from("Aristotle");

    let data = tests::wire::to_vec(&Struct { name: name.clone() })?;

    let struct_with_default: StructWithDefault = tests::wire::decode(data.as_slice())?;

    assert_eq!(
        struct_with_default,
        StructWithDefault {
            name: name.clone(),
            age: 0,
        }
    );

    let struct_with_option: StructWithOption = tests::wire::decode(data.as_slice())?;

    assert_eq!(struct_with_option, StructWithOption { name, age: None });

    Ok(())
}
