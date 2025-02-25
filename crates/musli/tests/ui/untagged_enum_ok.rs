use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(encode_only, untagged)]
enum UntaggedEnumOk {}

fn main() {}
