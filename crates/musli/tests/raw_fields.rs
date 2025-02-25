use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct RawFields {
    r#field: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum RawEnum {
    Variant { r#field: u32 },
}

#[test]
fn test_raw_fields() {
    musli::macros::assert_roundtrip_eq!(full, RawFields { r#field: 42 }, json = r#"{"field":42}"#);
    musli::macros::assert_roundtrip_eq!(
        full,
        RawEnum::Variant { r#field: 42 },
        json = r#"{"Variant":{"field":42}}"#
    );
}
