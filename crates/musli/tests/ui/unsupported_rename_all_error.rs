use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(name_all = "WHAT_IS_THIS")]
struct Struct {
    field: u32,
}

fn main() {
}

