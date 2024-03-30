#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
pub struct SkipSerialize {
    before: u32,
    #[musli(skip_encoding_if = Option::is_none)]
    skipped: Option<u32>,
    after: u32,
}

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
pub struct SkipSerializeUntagged {
    before: u32,
    #[musli(skip_encoding_if = Option::is_none)]
    skipped: Option<u32>,
    after: u32,
}

#[test]
fn skip_serialize() {
    tests::rt!(
        full,
        SkipSerializeUntagged {
            before: 1,
            skipped: Some(2),
            after: 3,
        },
        json = r#"[1,2,3]"#,
    );

    let out = tests::wire::transcode::<_, Unpacked>(SkipSerializeUntagged {
        before: 1,
        skipped: None,
        after: 3,
    });

    assert_eq!(out, Unpacked(1, 3));

    #[derive(Debug, PartialEq, Eq, Decode)]
    #[musli(packed)]
    struct Unpacked(u32, u32);
}
