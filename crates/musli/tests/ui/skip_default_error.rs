use musli::{Decode, Encode};

#[derive(Encode, Decode)]
struct WithSkipEncodingIf {
    #[musli(skip_default, skip_encoding_if = Option::is_none)]
    field: Option<u32>,
}

#[derive(Encode, Decode)]
struct WithSkip {
    #[musli(skip, skip_default)]
    field: u32,
}

#[derive(Encode, Decode)]
struct WithValue {
    #[musli(skip_default = default_field)]
    field: u32,
}

fn default_field() -> u32 {
    42
}

#[derive(Encode, Decode)]
#[musli(packed)]
struct Packed {
    first: u32,
    #[musli(skip_default)]
    last: u32,
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct Transparent {
    #[musli(skip_default)]
    field: u32,
}

#[derive(Default, Encode, Decode)]
struct NotPartialEq;

#[derive(Encode, Decode)]
struct MissingPartialEq {
    #[musli(skip_default)]
    field: NotPartialEq,
}

fn main() {}
