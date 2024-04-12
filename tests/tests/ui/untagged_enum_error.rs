use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed)]
enum UntaggedEnum {
}

fn main() {
}