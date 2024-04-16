use musli::{Encode, Decode};

#[test]
fn unit() {
    tests::rt!(full, ());
}

#[test]
fn tuples() {
    tests::rt!(full, (1, 2, 3, 4));
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleStruct(u32, u32);

#[test]
fn tuple_struct() {
    tests::rt!(full, TupleStruct(11, 13));
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Enum {
    Tuple(u32, u32),
}

#[test]
fn tuple_enum() {
    tests::rt!(full, Enum::Tuple(11, 13));
}
