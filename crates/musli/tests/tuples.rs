use musli::{Decode, Encode};

#[test]
fn unit() {
    musli::rt!(full, ());
}

#[test]
fn tuples() {
    musli::rt!(full, (1, 2, 3, 4));
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleStruct(u32, u32);

#[test]
fn tuple_struct() {
    musli::rt!(full, TupleStruct(11, 13));
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Enum {
    Tuple(u32, u32),
}

#[test]
fn tuple_enum() {
    musli::rt!(full, Enum::Tuple(11, 13));
}
