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
#[cfg(feature = "test")]
fn test_skip_serializing_if_outer() {
    tests::rt!(SkipSerializeOuter {
        flag: false,
        inner: Some(SkipSerializeInner),
    });

    tests::rt!(SkipSerializeOuter {
        flag: false,
        inner: None,
    });
}
