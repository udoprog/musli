#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
enum EmptyEnum {}

#[test]
fn decode_empty() {
    let e = musli::storage::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Type EmptyEnum cannot be decoded since it's uninhabitable"
    );

    let e = musli::wire::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Type EmptyEnum cannot be decoded since it's uninhabitable"
    );

    let e = musli::descriptive::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Type EmptyEnum cannot be decoded since it's uninhabitable"
    );

    let e = musli::json::from_slice::<EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "Type EmptyEnum cannot be decoded since it's uninhabitable"
    );
}
