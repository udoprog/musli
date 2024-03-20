#![cfg(feature = "test")]

use musli::{Decode, Encode};

/// Empty enums should work.
#[derive(Debug, Encode, Decode)]
enum EmptyEnum {}

#[test]
fn decode_empty() {
    let e = tests::wire::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");
}
