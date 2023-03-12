use musli::{Decode, Encode};

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[musli(bound = T: Encode<M>, decode_bound = T: Decode<'de, M>)]
pub struct GenericWithBound<T> {
    value: T,
}

#[test]
fn test_generic_with_bound() {
    musli_tests::rt!(
        GenericWithBound<String>,
        GenericWithBound {
            value: String::from("Hello"),
        }
    );
}
