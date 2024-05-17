use musli::{Decode, Encode};

#[test]
fn unit() {
    musli::macros::assert_roundtrip_eq!(full, ());
}

#[test]
fn tuples() {
    musli::macros::assert_roundtrip_eq!(full, (1, 2, 3, 4));
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleStruct(u32, u32);

#[test]
fn tuple_struct() {
    musli::macros::assert_roundtrip_eq!(full, TupleStruct(11, 13));
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Enum {
    Tuple(u32, u32),
}

#[test]
fn tuple_enum() {
    musli::macros::assert_roundtrip_eq!(full, Enum::Tuple(11, 13));
}
