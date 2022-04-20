use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Outer {
    pub flag: bool,
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub inner: Option<Inner>,
}

#[test]
fn test_skip_serializing_if_outer() {
    musli_wire::test::rt(Outer {
        flag: false,
        inner: Some(Inner),
    });

    musli_wire::test::rt(Outer {
        flag: false,
        inner: None,
    });
}
