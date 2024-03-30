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
    tests::rt!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: Some(SkipSerializeInner),
        },
        json = r#"{"0":false,"1":{}}"#,
    );

    tests::rt!(
        full,
        SkipSerializeOuter {
            flag: false,
            inner: None,
        },
        json = r#"{"0":false}"#,
    );
}
