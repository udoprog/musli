use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    field: u32,
    #[musli(default)]
    not_last: Option<u32>,
    last: u32,
}

#[derive(Encode, Decode)]
enum Enum {
    #[musli(packed)]
    Variant {
        field: u32,
        #[musli(default)]
        not_last: Option<u32>,
        last: u32,
    }
}

#[derive(Decode)]
#[musli(packed)]
enum EmptyEnum {
}

#[derive(Decode)]
#[musli(packed)]
enum EnumDecodePacked {
    Variant,
}

#[derive(Encode, Decode)]
#[musli(packed)]
struct DenyNamedPackedStruct {
    #[musli(name = "test")]
    field: String,
}

#[derive(Encode, Decode)]
#[musli(packed)]
struct DenyNamedPackedStruct(#[musli(name = "test")] String);

#[derive(Encode, Decode)]
#[musli(tag = "type")]
enum DenyNamedPackedEnum {
    #[musli(packed)]
    Struct {
        #[musli(name = "test")]
        field: String,
    },
    #[musli(packed)]
    Tuple(#[musli(name = "test")] String),
}

#[derive(Encode, Decode)]
#[musli(tag = "type")]
pub enum PackedVariant {
    #[musli(packed)]
    Variant(u32),
}

fn main() {
}
