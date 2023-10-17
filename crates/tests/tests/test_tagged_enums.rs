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
pub enum Variants {
    Variant1 { value: String },
    Variant2 { value: u32 },
}

#[test]
#[cfg(feature = "test")]
fn test_tagged_enums() {
    tests::rt!(EmptyVariants::Variant1);
    tests::rt!(EmptyVariants::Variant2);

    tests::rt!(TupleVariants::Variant1(String::from("foo")));
    tests::rt!(TupleVariants::Variant2(42));

    tests::rt!(Variants::Variant1 {
        value: String::from("foo"),
    });
    tests::rt!(Variants::Variant2 { value: 42 });
}
