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
    musli::macros::assert_roundtrip_eq!(full, EmptyVariants::Variant1);
    musli::macros::assert_roundtrip_eq!(full, EmptyVariants::Variant2);

    musli::macros::assert_roundtrip_eq!(full, TupleVariants::Variant1(String::from("foo")));
    musli::macros::assert_roundtrip_eq!(full, TupleVariants::Variant2(42));

    musli::macros::assert_roundtrip_eq!(
        full,
        Variants::Variant1 {
            value: String::from("foo"),
        }
    );
    musli::macros::assert_roundtrip_eq!(full, Variants::Variant2 { value: 42 });
}
