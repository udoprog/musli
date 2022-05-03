use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
struct FieldVariantTag<'a> {
    name: &'a str,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct BytesTag<'a>(&'a [u8]);

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct BytesTagVec(Vec<u8>);

/// A custom tag.
const CUSTOM_TAG1: FieldVariantTag = FieldVariantTag { name: "field1" };
const CUSTOM_TAG2: FieldVariantTag = FieldVariantTag { name: "field2" };

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructCustomFieldAsStruct {
    #[musli(rename = CUSTOM_TAG1)]
    field1: u32,
    #[musli(rename = CUSTOM_TAG2)]
    field2: u32,
}

#[test]
fn test_bytes_tag_vec() {
    musli_tests::rt!(BytesTagVec(b"hello world".to_vec()));
}

#[test]
fn test_custom_struct_tag() {
    musli_tests::rt!(StructCustomFieldAsStruct {
        field1: 42,
        field2: 84,
    });
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = [u8; 4])]
pub struct StructCustomTag {
    #[musli(rename = [1, 2, 3, 4])]
    field1: u32,
    #[musli(rename = [2, 3, 4, 5])]
    field2: u32,
}

#[test]
fn test_custom_tag() {
    musli_tests::rt!(StructCustomTag {
        field1: 42,
        field2: 84,
    });
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = BytesTag)]
struct StructWithBytesTag {
    #[musli(rename = BytesTag(b"name in bytes"))]
    string: String,
}

#[test]
fn test_struct_with_bytes_tag() {
    musli_tests::rt!(StructWithBytesTag {
        string: String::from("Some String"),
    });
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = BytesTag)]
enum EnumWithBytesTag {
    #[musli(rename = BytesTag(b"a"))]
    Variant1 { string: String },
    #[musli(rename = BytesTag(b"b"), name_type = BytesTag)]
    Variant2 {
        #[musli(rename = BytesTag(b"c"))]
        string: String,
    },
}

#[test]
fn test_bytes_tag_in_enum() {
    musli_tests::rt!(EnumWithBytesTag::Variant1 {
        string: String::from("st"),
    });

    musli_tests::rt!(EnumWithBytesTag::Variant2 {
        string: String::from("st"),
    });
}
