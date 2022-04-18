use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(variant = "name")]
enum EnumEmptyVariant {
    #[musli(name = "Other")]
    Something {},
}

#[test]
fn test_enum_with_empty_variant() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(EnumEmptyVariant::Something {})?;
    Ok(())
}
