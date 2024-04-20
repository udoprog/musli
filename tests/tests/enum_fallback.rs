#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = usize)]
pub enum Enum {
    Variant1,
    Variant2,
    Variant3,
    Variant4,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = usize)]
pub enum EnumDefault {
    #[musli(name = 3)]
    Variant4,
    #[musli(default)]
    Fallback,
}

#[test]
fn enum_default() {
    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant1,
        EnumDefault::Fallback,
        json = r#"{"0":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant2,
        EnumDefault::Fallback,
        json = r#"{"1":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant3,
        EnumDefault::Fallback,
        json = r#"{"2":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant4,
        EnumDefault::Variant4,
        json = r#"{"3":{}}"#,
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = usize)]
pub enum EnumPattern {
    Variant1,
    #[musli(pattern = 1..=2)]
    Fallback,
    #[musli(name = 3)]
    Variant4,
}

#[test]
fn enum_pattern() {
    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant1,
        EnumPattern::Variant1,
        json = r#"{"0":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant2,
        EnumPattern::Fallback,
        json = r#"{"1":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant3,
        EnumPattern::Fallback,
        json = r#"{"2":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        Enum::Variant4,
        EnumPattern::Variant4,
        json = r#"{"3":{}}"#,
    );
}
