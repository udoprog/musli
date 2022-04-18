use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
struct CustomStruct<'a> {
    name: &'a str,
}

/// A custom tag.
const CUSTOM_TAG1: CustomStruct = CustomStruct { name: "field1" };
const CUSTOM_TAG2: CustomStruct = CustomStruct { name: "field2" };

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructCustomFieldAsStruct {
    #[musli(tag = CUSTOM_TAG1)]
    field1: u32,
    #[musli(tag = CUSTOM_TAG2)]
    field2: u32,
}

#[test]
fn test_custom_struct_tag() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(StructCustomFieldAsStruct {
        field1: 42,
        field2: 84,
    })?;
    Ok(())
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag_type = [u8; 4])]
pub struct StructCustomTag {
    #[musli(tag = [1, 2, 3, 4])]
    field1: u32,
    #[musli(tag = [2, 3, 4, 5])]
    field2: u32,
}

#[test]
fn test_custom_tag() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(StructCustomTag {
        field1: 42,
        field2: 84,
    })?;
    Ok(())
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct BytesTag<'a>(&'a [u8]);

#[derive(Encode, Decode)]
#[musli(tag_type = BytesTag)]
struct StructWithBytesTag {
    #[musli(tag = BytesTag(r"name in bytes"))]
    string: String,
}

#[test]
fn test_bytes_tag() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(StructWithBytesTag {
        string: String::from("Some String"),
    })?;
    Ok(())
}
