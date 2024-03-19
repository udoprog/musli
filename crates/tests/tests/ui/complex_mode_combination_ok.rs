use musli::{Decode, Encode};

enum Packed {}

#[derive(Encode, Decode)]
#[musli(default_field = "name")]
#[musli(mode = Packed, encode_only, packed, default_field = "index")]
struct Person<'a> {
    name: &'a str,
    age: u32,
}

#[derive(Encode, Decode)]
#[musli(mode = Packed, encode_only, packed)]
enum Name<'a> {
    Full(&'a str),
    Given(&'a str),
}

fn main() {
}
