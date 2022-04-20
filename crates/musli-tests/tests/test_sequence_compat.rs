use musli::compat::Sequence;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SequenceCompat {
    pub empty_sequence: Sequence<()>,
}

#[test]
fn test_sequence_compat() {
    musli_tests::rt!(SequenceCompat {
        empty_sequence: Sequence(()),
    });
}
