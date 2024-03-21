use core::fmt;

use bstr::BStr;
use musli::compat::Bytes;
use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
struct FieldVariantTag<'a> {
    name: &'a str,
}

impl fmt::Display for FieldVariantTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct BytesTag<'a>(#[musli(bytes)] &'a [u8]);

impl fmt::Display for BytesTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(self.0).fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct BytesTagVec(Vec<u8>);

impl fmt::Display for BytesTagVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(&self.0).fmt(f)
    }
}

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
fn bytes_tag_vec() {
    tests::rt!(BytesTagVec(b"hello world".to_vec()));
}

#[test]
fn custom_struct_tag() {
    tests::rt!(StructCustomFieldAsStruct {
        field1: 42,
        field2: 84,
    });
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = Bytes<[u8; 4]>, name_format_with = BStr::new)]
pub struct StructCustomTag {
    #[musli(rename = Bytes([1, 2, 3, 4]))]
    field1: u32,
    #[musli(rename = Bytes([2, 3, 4, 5]))]
    field2: u32,
}

#[test]
fn custom_tag() {
    tests::rt!(StructCustomTag {
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
fn struct_with_bytes_tag() {
    tests::rt!(StructWithBytesTag {
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
fn bytes_tag_in_enum() {
    tests::rt!(EnumWithBytesTag::Variant1 {
        string: String::from("st"),
    });

    tests::rt!(EnumWithBytesTag::Variant2 {
        string: String::from("st"),
    });
}
