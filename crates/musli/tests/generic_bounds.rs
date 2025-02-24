use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[musli(Binary, bound = {T: Encode<Binary>}, decode_bound<'de, A> = {T: Decode<'de, Binary, A>})]
#[musli(Text, bound = {T: Encode<Text>}, decode_bound<'de, A> = {T: Decode<'de, Text, A>})]
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
