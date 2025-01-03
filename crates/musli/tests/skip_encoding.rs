#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
pub struct SkipEncodeIf {
    before: u32,
    #[musli(default, skip_encoding_if = Option::is_none)]
    skipped: Option<u32>,
    after: u32,
}

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
pub struct SkipEncodeIfPacked {
    before: u32,
    #[musli(skip_encoding_if = Option::is_none)]
    skipped: Option<u32>,
    after: u32,
}

#[derive(Debug, PartialEq, Eq, Decode)]
#[musli(packed)]
struct SkipEncodeIfPackedRepr(u32, u32);

#[test]
fn skip_serialize() {
    musli::macros::assert_roundtrip_eq!(
        full,
        SkipEncodeIf {
            before: 1,
            skipped: Some(2),
            after: 3,
        },
        json = r#"{"before":1,"skipped":2,"after":3}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        SkipEncodeIf {
            before: 1,
            skipped: None,
            after: 3,
        },
        json = r#"{"before":1,"after":3}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        SkipEncodeIfPacked {
            before: 1,
            skipped: Some(2),
            after: 3,
        },
        json = r#"[1,2,3]"#,
    );

    musli::macros::assert_decode_eq! {
        full,
        SkipEncodeIfPacked {
            before: 1,
            skipped: None,
            after: 3,
        },
        SkipEncodeIfPackedRepr(1, 3),
        json = r#"[1,3]"#,
    };
}
