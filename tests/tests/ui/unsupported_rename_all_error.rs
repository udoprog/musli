use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(rename_all = "WHAT_IS_THIS")]
struct Struct {
    field: u32,
}

fn main() {
}

