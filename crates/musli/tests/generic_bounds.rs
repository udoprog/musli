use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[musli(mode = Binary, bound = {T: Encode<Binary>}, decode_bound = {T: Decode<'de, Binary>})]
#[musli(mode = Text, bound = {T: Encode<Text>}, decode_bound = {T: Decode<'de, Text>})]
pub struct GenericWithBound<T> {
    value: T,
}

#[test]
fn generic_with_bound() {
    musli::macros::assert_roundtrip_eq!(
        full,
        GenericWithBound {
            value: String::from("Hello"),
        }
    );
}
