#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SkipSerializeInner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SkipSerializeOuter {
    pub flag: bool,
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub inner: Option<SkipSerializeInner>,
}

#[test]
fn skip_serializing_if_outer() {
    musli::macros::assert_roundtrip_eq!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: Some(SkipSerializeInner),
        },
        json = r#"{"flag":false,"inner":{}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: None,
        },
        json = r#"{"flag":false}"#,
    );
}
