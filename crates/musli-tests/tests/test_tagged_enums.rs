use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum TaggedEnum1 {
    Variant1,
    Variant2,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum TaggedEnum2 {
    Variant1(String),
    Variant2(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum TaggedEnum3 {
    Variant1 { value: String },
    Variant2 { value: u32 },
}

#[test]
fn test_tagged_enums() {
    musli_tests::rt!(TaggedEnum1::Variant1);
    musli_tests::rt!(TaggedEnum1::Variant2);

    musli_tests::rt!(TaggedEnum2::Variant1(String::from("foo")));
    musli_tests::rt!(TaggedEnum2::Variant2(42));

    musli_tests::rt!(TaggedEnum3::Variant1 {
        value: String::from("foo"),
    });
    musli_tests::rt!(TaggedEnum3::Variant2 { value: 42 });
}
