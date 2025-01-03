use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Struct {
    field: u32,
    #[musli(default)]
    not_last: Option<u32>,
    last: u32,
}

fn main() {
}
