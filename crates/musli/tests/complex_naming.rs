use core::fmt;
use core::ops::Deref;

use bstr::BStr;
use musli::compat::Bytes;
use musli::de::{DecodeUnsized, Decoder};
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
struct CustomBytesVec(Vec<u8>);

impl fmt::Display for CustomBytesVec {
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
    musli::rt!(full, CustomBytesVec(b"hello world".to_vec()));
}

#[test]
fn custom_struct_tag() {
    musli::rt!(
        no_json,
        StructCustomFieldAsStruct {
            field1: 42,
            field2: 84,
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = Bytes<[u8; 4]>, name_format_with = BStr::new)]
pub struct BytesName {
    #[musli(name = Bytes([1, 2, 3, 4]))]
    field1: u32,
    #[musli(name = Bytes([2, 3, 4, 5]))]
    field2: u32,
}

#[test]
fn bytes_name() {
    musli::rt!(
        no_json,
        BytesName {
            field1: 42,
            field2: 84,
        }
    );
}

#[derive(Encode)]
#[repr(transparent)]
#[musli(transparent)]
struct UnsizedBytes(#[musli(bytes)] [u8]);

impl<'de, M> DecodeUnsized<'de, M> for UnsizedBytes {
    fn decode_unsized<D, F, O>(_: &D::Cx, decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        decoder.decode_unsized_bytes(|bytes: &[u8]| f(UnsizedBytes::new(bytes)))
    }
}

impl UnsizedBytes {
    const fn new(data: &[u8]) -> &Self {
        // SAFETY: `UnsizedBytes` is a transparent wrapper around `[u8]`.
        unsafe { &*(data as *const [u8] as *const UnsizedBytes) }
    }
}

impl Deref for UnsizedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for UnsizedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(&self.0).fmt(f)
    }
}

impl fmt::Debug for UnsizedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(&self.0).fmt(f)
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = UnsizedBytes, name_method = "unsized")]
pub struct StructUnsizedBytes {
    #[musli(name = UnsizedBytes::new(&[1, 2, 3, 4]), pattern = UnsizedBytes([1, 2, 3, 4]))]
    field1: u32,
    #[musli(name = UnsizedBytes::new(&[2, 3, 4, 5]), pattern = UnsizedBytes([2, 3, 4, 5]))]
    field2: u32,
}

#[test]
fn struct_unsized_bytes() {
    musli::rt!(
        no_json,
        StructUnsizedBytes {
            field1: 42,
            field2: 84,
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = UnsizedBytes, name_method = "unsized")]
pub enum EnumUnsizedBytes {
    #[musli(name = UnsizedBytes::new(&[1, 2, 3, 4]), pattern = UnsizedBytes([1, 2, 3, 4]))]
    Variant1 { field1: u32 },
    #[musli(name = UnsizedBytes::new(&[2, 3, 4, 5]), pattern = UnsizedBytes([2, 3, 4, 5]))]
    Variant2 { field2: u32 },
}

#[test]
fn enum_unsized_bytes() {
    musli::rt!(no_json, EnumUnsizedBytes::Variant1 { field1: 42 });
    musli::rt!(no_json, EnumUnsizedBytes::Variant2 { field2: 84 });
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(transparent)]
struct CustomBytes<'a>(#[musli(bytes)] &'a [u8]);

impl fmt::Display for CustomBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BStr::new(self.0).fmt(f)
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = CustomBytes)]
struct StructWithCustomBytes {
    #[musli(name = CustomBytes(b"name in bytes"))]
    string: String,
}

#[test]
fn struct_with_bytes_name() {
    musli::rt!(
        no_json,
        StructWithCustomBytes {
            string: String::from("Some String"),
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = CustomBytes)]
enum EnumWithCustomBytes {
    #[musli(name = CustomBytes(b"a"))]
    Variant1 { string: String },
    #[musli(name = CustomBytes(b"b"), name_type = CustomBytes)]
    Variant2 {
        #[musli(name = CustomBytes(b"c"))]
        string: String,
    },
}

#[test]
fn bytes_tag_in_enum() {
    musli::rt!(
        no_json,
        EnumWithCustomBytes::Variant1 {
            string: String::from("st"),
        }
    );

    musli::rt!(
        no_json,
        EnumWithCustomBytes::Variant2 {
            string: String::from("st"),
        }
    );
}
