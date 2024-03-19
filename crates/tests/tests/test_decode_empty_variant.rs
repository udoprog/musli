use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_variant = "name")]
enum EnumEmptyVariant {
    #[musli(rename = "Other")]
    Something {},
}

#[test]
#[cfg(feature = "test")]
fn test_enum_with_empty_variant() {
    tests::rt!(EnumEmptyVariant::Something {});
}
