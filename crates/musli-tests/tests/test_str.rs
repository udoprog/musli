use anyhow::Result;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithStr<'a> {
    name: &'a str,
    age: u32,
}

#[test]
fn test_deserialize_roundtrip() -> Result<()> {
    let data = musli_tests::wire::to_vec(&StructWithStr {
        name: "Jane Doe",
        age: 42,
    })?;

    let with_str: StructWithStr<'_> = musli_tests::wire::decode(&data[..])?;
    assert_eq!(with_str.name, "Jane Doe");
    assert_eq!(with_str.age, 42);
    Ok(())
}
