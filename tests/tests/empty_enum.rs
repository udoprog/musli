#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
enum EmptyEnum {}

#[test]
fn decode_empty() {
    let e = musli::storage::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = musli::wire::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = musli::descriptive::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");

    let e = musli::json::from_slice::<EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(e.to_string(), "Cannot decode uninhabitable types");
}
