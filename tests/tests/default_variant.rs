#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum SeveralVariants {
    Variant1,
    Variant2,
    Variant3,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub enum OnlyFallback {
    // Renamed to something which is not provided in `SeveralVariants`.
    #[musli(name = 4)]
    Variant4,
    #[musli(default)]
    Fallback,
}

/// Test that enums can use fallback variants when decoding.
#[test]
fn fallback_variant() {
    tests::assert_decode_eq!(
        upgrade_stable,
        SeveralVariants::Variant1,
        OnlyFallback::Fallback,
        json = r#"{"0":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        SeveralVariants::Variant2,
        OnlyFallback::Fallback,
        json = r#"{"1":{}}"#,
    );

    tests::assert_decode_eq!(
        upgrade_stable,
        SeveralVariants::Variant3,
        OnlyFallback::Fallback,
        json = r#"{"2":{}}"#,
    );
}
