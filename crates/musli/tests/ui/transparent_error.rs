use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentStruct {
    first: u32,
    second: u32,
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentTuple(u32, u32);

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentEmptyStruct;

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentEmptyTuple;

#[derive(Encode, Decode)]
enum Enum1 {
    #[musli(transparent)]
    Variant {
        first: u32,
        second: u32,
    },
    #[musli(transparent)]
    TransparentTuple(u32, u32),
    #[musli(transparent)]
    TransparentEmptyStruct,
    #[musli(transparent)]
    TransparentEmptyTuple,
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct DenyNamedTransparentStruct {
    #[musli(name = "test")]
    field: String,
}

#[derive(Encode, Decode)]
#[musli(tag = "type")]
enum DenyNamedTransparentEnum {
    #[musli(transparent)]
    Variant(#[musli(name = "test")] String),
}

#[derive(Encode, Decode)]
#[musli(packed)]
struct DenyNamedPackedStruct {
    #[musli(name = "test")]
    field: String,
}

#[derive(Encode, Decode)]
#[musli(tag = "type")]
enum DenyNamedPackedEnum {
    #[musli(packed)]
    Variant(#[musli(name = "test")] String),
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct DenyOptionalTransparentStruct {
    #[musli(skip_encoding_if = String::is_empty)]
    field: String,
}

#[derive(Encode, Decode)]
enum DenyOptionalTransparentEnum {
    #[musli(transparent)]
    Variant {
        #[musli(skip_encoding_if = String::is_empty)]
        field: String,
    }
}

#[derive(Encode, Decode)]
#[musli(tag = "type")]
pub enum PackedVariant {
    #[musli(packed)]
    Variant(u32),
}

fn main() {
}
