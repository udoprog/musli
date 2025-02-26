use musli::{Decode, Encode};

enum Packed {}

#[derive(Encode, Decode)]
#[musli(mode = Packed, encode_only, packed)]
#[musli(mode = Packed, decode_only, packed)]
struct Person<'a> {
    name: &'a str,
    age: u32,
}

#[derive(Encode, Decode)]
#[musli(mode = Packed, encode_only, untagged)]
enum Name<'a> {
    Full(&'a str),
    Given(&'a str),
}

fn main() {}
