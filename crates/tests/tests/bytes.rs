use std::collections::VecDeque;

use musli::compat::Bytes;
use musli::{Decode, Encode};

#[derive(Decode, Encode)]
struct Container<'de> {
    #[musli(bytes)]
    vec: Vec<u8>,
    #[musli(bytes)]
    vec_deque: VecDeque<u8>,
    #[musli(bytes)]
    bytes: &'de [u8],
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct BytesCompat {
    pub empty_bytes: Bytes<[u8; 0]>,
}

#[test]
fn bytes_compat() {
    tests::rt!(BytesCompat {
        empty_bytes: Bytes([]),
    });
}
