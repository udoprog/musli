use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct EmptyStruct;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct2(String);

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct3(String, u32);

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct4 {
    value: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Struct5 {
    value: String,
    value2: u32,
}

#[test]
#[cfg(feature = "test")]
fn test_struct() {
    tests::rt!(EmptyStruct);
    tests::rt!(Struct2(String::from("Hello World")));
    tests::rt!(Struct3(String::from("Hello World"), 42));
    tests::rt!(Struct4 {
        value: String::from("Hello World"),
    });
    tests::rt!(Struct5 {
        value: String::from("Hello World"),
        value2: 42,
    });
}
