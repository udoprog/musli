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
fn test_skip_serialize() {
    musli_wire::test::rt(SkipSerializeUntagged {
        before: 1,
        skipped: Some(2),
        after: 3,
    });

    let out = musli_wire::test::transcode::<_, (u32, u32)>(SkipSerializeUntagged {
        before: 1,
        skipped: None,
        after: 3,
    });

    assert_eq!(out, (1, 3));
}
