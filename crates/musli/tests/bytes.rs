use std::collections::VecDeque;

use musli::compat::Bytes;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
struct Container {
    #[musli(bytes)]
    vec: Vec<u8>,
    #[musli(bytes)]
    vec_deque: VecDeque<u8>,
    #[musli(bytes)]
    boxed: Box<[u8]>,
}

#[test]
fn container() {
    musli::macros::assert_roundtrip_eq!(
        full,
        Container {
            vec: vec![0, 1, 2, 3],
            vec_deque: VecDeque::from([0, 1, 2, 3]),
            boxed: Box::from([0, 1, 2, 3]),
        },
        json = r#"{"vec":[0,1,2,3],"vec_deque":[0,1,2,3],"boxed":[0,1,2,3]}"#
    );
}

#[derive(Debug, PartialEq, Decode, Encode)]
#[allow(dead_code)]
struct ContainerBorrowed<'de> {
    #[musli(bytes)]
    bytes: &'de [u8],
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct BytesCompat {
    pub empty_bytes: Bytes<[u8; 0]>,
}

#[test]
fn bytes_compat() {
    musli::macros::assert_roundtrip_eq!(
        full,
        BytesCompat {
            empty_bytes: Bytes([]),
        }
    );
}
