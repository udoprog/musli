use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_variant_tag = "name")]
enum EnumEmptyVariant {
    #[musli(tag = "Other")]
    Something {},
}

#[test]
fn test_enum_with_empty_variant() {
    musli_wire::test::rt(EnumEmptyVariant::Something {});
}
