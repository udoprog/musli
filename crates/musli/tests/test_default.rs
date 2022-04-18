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
fn test_decode_with_default() -> Result<(), Box<dyn std::error::Error>> {
    let name = String::from("Aristotle");

    let data = musli_wire::to_vec(&Struct { name: name.clone() })?;

    let struct_with_default: StructWithDefault = musli_wire::decode(&data[..])?;

    assert_eq!(
        struct_with_default,
        StructWithDefault {
            name: name.clone(),
            age: 0,
        }
    );

    let struct_with_option: StructWithOption = musli_wire::decode(&data[..])?;

    assert_eq!(
        struct_with_option,
        StructWithOption {
            name: name.clone(),
            age: None,
        }
    );

    Ok(())
}
