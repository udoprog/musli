use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed, transparent)]
struct Struct {
    field: u32,
}

fn main() {
}