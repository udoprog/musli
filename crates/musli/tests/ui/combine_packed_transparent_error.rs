use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(packed, transparent)]
struct Struct {
    field: u32,
}

fn main() {}
