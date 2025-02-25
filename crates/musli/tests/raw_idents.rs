use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct {
    r#field: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "PascalCase")]
struct StructRename {
    r#field_name: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructOverride {
    #[musli(Text, name = "r#field")]
    r#field: u32,
}

#[test]
fn struct_idents() {
    musli::macros::assert_roundtrip_eq!(full, Struct { r#field: 42 }, json = r#"{"field":42}"#);
    musli::macros::assert_roundtrip_eq!(
        full,
        StructRename { r#field_name: 42 },
        json = r#"{"FieldName":42}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        StructOverride { r#field: 42 },
        json = r#"{"r#field":42}"#
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Enum {
    Variant { r#field: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum EnumVariant {
    r#Variant { r#field: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum r#EnumField {
    r#Variant { r#field: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum r#EnumOverride {
    #[musli(Text, name = "r#Variant")]
    r#Variant { r#field: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "snake_case")]
enum r#EnumRawRename {
    r#VariantRename { r#field: u32 },
}

#[test]
fn enum_idents() {
    musli::macros::assert_roundtrip_eq!(
        full,
        Enum::Variant { r#field: 42 },
        json = r#"{"Variant":{"field":42}}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        EnumVariant::r#Variant { r#field: 42 },
        json = r#"{"Variant":{"field":42}}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        r#EnumField::r#Variant { r#field: 42 },
        json = r#"{"Variant":{"field":42}}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        r#EnumOverride::r#Variant { r#field: 42 },
        json = r#"{"r#Variant":{"field":42}}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        r#EnumRawRename::r#VariantRename { r#field: 42 },
        json = r#"{"variant_rename":{"field":42}}"#
    );
}
