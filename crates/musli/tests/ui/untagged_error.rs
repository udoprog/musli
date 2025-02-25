use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(untagged)]
struct Struct1;

#[derive(Encode, Decode)]
#[musli(untagged)]
struct Struct2();

#[derive(Encode, Decode)]
#[musli(untagged)]
struct Struct3 {}

#[derive(Encode, Decode)]
#[musli(untagged)]
enum Enum1 {}

#[derive(Encode, Decode)]
#[musli(untagged)]
enum Enum2 {
    #[musli(untagged)]
    Variant1,
    #[musli(untagged)]
    Variant2(),
    #[musli(untagged)]
    Variant3 {},
}

fn main() {}
