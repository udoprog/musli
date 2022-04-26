use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum EmptyVariants {
    Variant1,
    Variant2,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum TupleVariants {
    Variant1(String),
    Variant2(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum StructVariants {
    Variant1 { value: String },
    Variant2 { value: u32 },
}

#[test]
fn test_tagged_enums() {
    musli_tests::rt!(EmptyVariants::Variant1);
    musli_tests::rt!(EmptyVariants::Variant2);

    musli_tests::rt!(TupleVariants::Variant1(String::from("foo")));
    musli_tests::rt!(TupleVariants::Variant2(42));

    musli_tests::rt!(StructVariants::Variant1 {
        value: String::from("foo"),
    });
    musli_tests::rt!(StructVariants::Variant2 { value: 42 });
}
