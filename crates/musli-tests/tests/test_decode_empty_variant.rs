use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_variant_name = "name")]
enum EnumEmptyVariant {
    #[musli(rename = "Other")]
    Something {},
}

#[test]
fn test_enum_with_empty_variant() {
    musli_tests::rt!(EnumEmptyVariant::Something {});
}
