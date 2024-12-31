#![allow(dead_code)]
#![allow(clippy::byte_char_slices)]

use bstr::BStr;
use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(name(type = str))]
struct StructStr {
    #[musli(name = "field1")]
    field1: u32,
    #[musli(name = "field2")]
    field2: u32,
}

#[derive(Encode, Decode)]
#[musli(name(type = [u8], format_with = BStr::new))]
struct StructBytes {
    #[musli(name = &[b'f', b'i', b'e', b'l', b'd', b'1'])]
    field1: u32,
    #[musli(name = &[b'f', b'i', b'e', b'l', b'd', b'2'])]
    field2: u32,
}

#[derive(Encode, Decode)]
#[musli(name(type = [u8], format_with = BStr::new))]
struct StructBytesArray {
    #[musli(name = b"field1")]
    field1: u32,
    #[musli(name = b"field2")]
    field2: u32,
}
