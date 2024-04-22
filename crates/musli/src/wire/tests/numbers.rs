#![cfg(feature = "test")]

use crate::wire::tag::{Kind, Tag};
use crate::wire::test::Typed;
use crate::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[musli(crate)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
    e: i128,
}

#[derive(Debug, Clone, PartialEq, Decode)]
#[musli(crate, packed)]
struct Unpacked {
    count: Tag,
    #[musli(packed)]
    a: (Tag, Tag),
    #[musli(packed)]
    b: (Tag, Tag),
    #[musli(packed)]
    c: (Tag, Tag),
    #[musli(packed)]
    d: (Tag, Tag),
    #[musli(packed)]
    e: (Tag, Typed<5>),
}

#[test]
fn signed_unpacked() {
    let out = StructWithNumbers {
        a: -1,
        b: 1,
        c: -2,
        d: 2,
        e: 10000000000,
    };

    let out = crate::wire::to_vec(&out).expect("failed to encode");
    let unpacked: Unpacked = crate::storage::decode(out.as_slice()).expect("failed to decode");

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
