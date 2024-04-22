use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentStruct {
    first: u32,
    second: u32,
}

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentTuple(u32, u32);

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentEmptyStruct;

#[derive(Encode, Decode)]
#[musli(transparent)]
struct TransparentEmptyTuple;

#[derive(Encode, Decode)]
enum Enum1 {
    #[musli(transparent)]
    Variant {
        first: u32,
        second: u32,
    },
    #[musli(transparent)]
    TransparentTuple(u32, u32),
    #[musli(transparent)]
    TransparentEmptyStruct,
    #[musli(transparent)]
    TransparentEmptyTuple,
}

fn main() {
}