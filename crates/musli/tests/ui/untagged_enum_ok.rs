use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(encode_only, packed)]
enum UntaggedEnumOk {}

fn main() {}
