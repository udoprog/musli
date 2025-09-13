use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(Text, tag = "type")]
pub enum Enum<'de> {
    Variant { field: &'de str },
}
