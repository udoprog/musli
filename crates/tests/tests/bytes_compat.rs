use musli::compat::Bytes;
use musli::{Decode, Encode};

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
