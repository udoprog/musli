use musli::{Decode, Encode};
use musli_tests::wire::tag::{Kind, Tag};
use musli_tests::wire::Typed;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
    e: i128,
}

#[test]
fn test_signed_unpacked() {
    let out = musli_tests::rt!(StructWithNumbers {
        a: -1,
        b: 1,
        c: -2,
        d: 2,
        e: 10000000000,
    });

    let out = musli_tests::wire::to_vec(&out).expect("failed to encode");
    let unpacked: Unpacked = musli_tests::storage::decode(&out[..]).expect("failed to decode");

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

    #[derive(Debug, Clone, PartialEq, Decode)]
    #[musli(packed)]
    struct Unpacked {
        count: Tag,
        a: (Tag, Tag),
        b: (Tag, Tag),
        c: (Tag, Tag),
        d: (Tag, Tag),
        e: (Tag, Typed<[u8; 5]>),
    }
}
