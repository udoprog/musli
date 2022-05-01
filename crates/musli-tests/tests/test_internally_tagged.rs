use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type")]
pub enum InternallyTagged {}

#[test]
fn test_internally_tagged() {}
