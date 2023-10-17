use musli::{Decode, Encode};

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[musli(bound = {T: Encode<M>}, decode_bound = {T: Decode<'de, M>})]
pub struct GenericWithBound<T> {
    value: T,
}

#[test]
#[cfg(feature = "test")]
fn test_generic_with_bound() {
    tests::rt!(
        GenericWithBound<String>,
        GenericWithBound {
            value: String::from("Hello"),
        }
    );
}
