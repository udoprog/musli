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
fn test_skip_serializing_if_outer() {
    musli_tests::rt!(SkipSerializeOuter {
        flag: false,
        inner: Some(SkipSerializeInner),
    });

    musli_tests::rt!(SkipSerializeOuter {
        flag: false,
        inner: None,
    });
}
