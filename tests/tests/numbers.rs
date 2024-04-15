#![cfg(feature = "test")]

use musli::{Decode, Encode};
use tests::wire::tag::{Kind, Tag};
use tests::wire::Typed;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
    e: i128,
}

#[derive(Debug, Clone, PartialEq, Decode)]
#[musli(packed)]
struct Unpacked {
    count: Tag,
    a: (Tag, Tag),
    b: (Tag, Tag),
    c: (Tag, Tag),
    d: (Tag, Tag),
    e: (Tag, Typed<5>),
}

#[test]
fn signed_unpacked() {
    let out = tests::rt!(
        full,
        StructWithNumbers {
            a: -1,
            b: 1,
            c: -2,
            d: 2,
            e: 10000000000,
        }
    );

    let out = tests::wire::to_vec(&out).expect("failed to encode");
    let unpacked: Unpacked = tests::storage::decode(out.as_slice()).expect("failed to decode");

    assert_eq! {
        unpacked,
        Unpacked {
            count: Tag::new(Kind::Sequence, 10),
            a: (Tag::new(Kind::Continuation, 0), Tag::new(Kind::Continuation, 1)),
            b: (Tag::new(Kind::Continuation, 1), Tag::new(Kind::Continuation, 2)),
            c: (Tag::new(Kind::Continuation, 2), Tag::new(Kind::Continuation, 3)),
            d: (Tag::new(Kind::Continuation, 3), Tag::new(Kind::Continuation, 4)),
            e: (Tag::new(Kind::Continuation, 4), Typed::new(Tag::empty(Kind::Continuation), [128, 144, 223, 192, 74])),
        }
    };
}
