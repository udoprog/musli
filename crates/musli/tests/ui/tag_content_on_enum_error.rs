use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(tag = "type")]
struct TagUnsupported;

#[derive(Encode, Decode)]
#[musli(content = "type")]
struct ContentUnsupported;

fn main() {
}
