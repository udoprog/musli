use std::collections::VecDeque;

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
struct PackedFields {
    #[musli(packed)]
    tuple: (u16, u32, u64, u128),
    #[musli(packed)]
    array: [u32; 4],
}

#[test]
fn packed_fields() {
    musli::macros::assert_roundtrip_eq!(
        full,
        PackedFields {
            tuple: (11, 13, 15, 17),
            array: [11, 13, 15, 17],
        },
        json = r#"{"tuple":[11,13,15,17],"array":[11,13,15,17]}"#
    );
}

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
struct PackedVec {
    #[musli(packed)]
    data: Vec<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct UnpackedVec {
    #[musli(packed)]
    data: [u32; 4],
}

#[test]
fn packed_vec() {
    musli::macros::assert_decode_eq! {
        full,
        PackedVec { data: vec![u32::MIN, u32::MAX, 0, 10] },
        UnpackedVec { data: [u32::MIN, u32::MAX, 0, 10] }
    };
}

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
struct PackedVecDeque {
    #[musli(packed)]
    data: VecDeque<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct UnpackedVecDeque {
    #[musli(packed)]
    data: [u32; 4],
}

#[test]
fn packed_vec_deque() {
    musli::macros::assert_decode_eq! {
        full,
        PackedVecDeque { data: VecDeque::from(vec![u32::MIN, u32::MAX, 0, 10]) },
        UnpackedVecDeque { data: [u32::MIN, u32::MAX, 0, 10] }
    };
}
