use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum UntaggedVariants {
    #[musli(packed)]
    Empty,
    #[musli(packed)]
    Tuple(u32, u32),
    #[musli(packed)]
    Struct { a: u32, b: u32 },
}

/// Enums may contain packed variants.
#[test]
fn test_untagged_variants() {
    musli_tests::rt!(UntaggedVariants::Empty);
    musli_tests::rt!(UntaggedVariants::Tuple(42, 84));
    musli_tests::rt!(UntaggedVariants::Struct { a: 42, b: 84 });
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
enum UntaggedSingleFields {
    #[musli(packed)]
    Variant1(String),
    #[musli(packed)]
    Variant2(f64),
    #[musli(packed)]
    Variant3(u64),
}

#[test]
fn test_untagged_single_field_variant() {
    musli_tests::rt!(UntaggedSingleFields::Variant1(String::from("hello")));
    musli_tests::rt!(UntaggedSingleFields::Variant2(1.0));
    musli_tests::rt!(UntaggedSingleFields::Variant3(42));
}
