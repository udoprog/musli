#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
enum EmptyEnum {}

#[test]
fn decode_empty() {
    let e = tests::storage::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = tests::wire::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = tests::descriptive::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = tests::json::from_slice::<EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");
}
