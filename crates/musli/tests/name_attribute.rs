use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name(type = usize, method = "sized"))]
pub struct Struct2 {
    field1: u32,
    field2: u32,
    field3: u32,
    field4: u32,
}
