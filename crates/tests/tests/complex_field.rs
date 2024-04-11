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
#[musli(name_type = FieldVariantTag)]
pub struct StructCustomFieldAsStruct {
    #[musli(name = CUSTOM_TAG1)]
    field1: u32,
    #[musli(name = CUSTOM_TAG2)]
    field2: u32,
}

#[test]
fn bytes_tag_vec() {
    tests::rt!(full, BytesTagVec(b"hello world".to_vec()));
}

#[test]
fn custom_struct_tag() {
    tests::rt!(
        no_json,
        StructCustomFieldAsStruct {
            field1: 42,
            field2: 84,
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = Bytes<[u8; 4]>, name_format_with = BStr::new)]
pub struct StructCustomTag {
    #[musli(name = Bytes([1, 2, 3, 4]))]
    field1: u32,
    #[musli(name = Bytes([2, 3, 4, 5]))]
    field2: u32,
}

#[test]
fn custom_tag() {
    tests::rt!(
        no_json,
        StructCustomTag {
            field1: 42,
            field2: 84,
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = BytesTag)]
struct StructWithBytesTag {
    #[musli(name = BytesTag(b"name in bytes"))]
    string: String,
}

#[test]
fn struct_with_bytes_tag() {
    tests::rt!(
        no_json,
        StructWithBytesTag {
            string: String::from("Some String"),
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = BytesTag)]
enum EnumWithBytesTag {
    #[musli(name = BytesTag(b"a"))]
    Variant1 { string: String },
    #[musli(name = BytesTag(b"b"), name_type = BytesTag)]
    Variant2 {
        #[musli(name = BytesTag(b"c"))]
        string: String,
    },
}

#[test]
fn bytes_tag_in_enum() {
    tests::rt!(
        no_json,
        EnumWithBytesTag::Variant1 {
            string: String::from("st"),
        }
    );

    tests::rt!(
        no_json,
        EnumWithBytesTag::Variant2 {
            string: String::from("st"),
        }
    );
}
