use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "name")]
enum EmptyVariants {
    Empty,
    Tuple(),
    Struct {},
}

#[test]
fn enum_with_empty_variant() {
    musli::macros::assert_roundtrip_eq!(full, EmptyVariants::Empty, json = r#"{"Empty":{}}"#);
    musli::macros::assert_roundtrip_eq!(full, EmptyVariants::Tuple(), json = r#"{"Tuple":{}}"#);
    musli::macros::assert_roundtrip_eq!(full, EmptyVariants::Struct {}, json = r#"{"Struct":{}}"#);
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "name")]
enum NamedVariants {
    #[musli(name_all = "index")]
    Variant1 { field: u32 },
    #[musli(name_all = "index")]
    Variant2 { field: u32 },
}

#[test]
fn multiple_named_variants() {
    musli::macros::assert_roundtrip_eq!(
        full,
        NamedVariants::Variant1 { field: 1 },
        json = r#"{"Variant1":{"0":1}}"#
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        NamedVariants::Variant2 { field: 2 },
        json = r#"{"Variant2":{"0":2}}"#
    );
}

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
fn untagged_variants() {
    musli::macros::assert_roundtrip_eq!(full, UntaggedVariants::Empty, json = r#"{"Empty":[]}"#);
    musli::macros::assert_roundtrip_eq!(
        full,
        UntaggedVariants::Tuple(42, 84),
        json = r#"{"Tuple":[42,84]}"#
    );
    musli::macros::assert_roundtrip_eq!(
        full,
        UntaggedVariants::Struct { a: 42, b: 84 },
        json = r#"{"Struct":[42,84]}"#
    );
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
fn untagged_single_field_variant() {
    musli::macros::assert_roundtrip_eq!(
        full,
        UntaggedSingleFields::Variant1(String::from("hello"))
    );
    musli::macros::assert_roundtrip_eq!(full, UntaggedSingleFields::Variant2(1.0));
    musli::macros::assert_roundtrip_eq!(full, UntaggedSingleFields::Variant3(42));
}
