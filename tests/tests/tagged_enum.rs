#![cfg(feature = "test")]

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
fn tagged_enums() {
    tests::rt!(full, EmptyVariants::Variant1);
    tests::rt!(full, EmptyVariants::Variant2);

    tests::rt!(full, TupleVariants::Variant1(String::from("foo")));
    tests::rt!(full, TupleVariants::Variant2(42));

    tests::rt!(
        full,
        Variants::Variant1 {
            value: String::from("foo"),
        }
    );
    tests::rt!(full, Variants::Variant2 { value: 42 });
}
