use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(encode_only, packed)]
enum UntaggedEnumOk {
}

fn main() {
}
