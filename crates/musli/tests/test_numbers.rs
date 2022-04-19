use musli::{Decode, Encode};
use musli_wire::{
    test::Typed,
    types::{Kind, Tag, CONTINUATION},
};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
}

#[test]
fn test_signed_unpacked() {
    let out = musli_wire::test::rt(StructWithNumbers {
        a: -1,
        b: 1,
        c: -3,
        d: 3,
    });

    let out = musli_wire::to_vec(&out).expect("failed to encode");

    let unpacked: Unpacked = musli_storage::decode(&out[..]).expect("failed to decode");

    assert_eq! {
        unpacked,
        Unpacked {
            count: Tag::new(Kind::PairSequence, 4),
            a_tag: Typed::new(CONTINUATION, 0),
            a: Typed::new(CONTINUATION, 1),
            b_tag: Typed::new(CONTINUATION, 1),
            b: Typed::new(CONTINUATION, 2),
            c_tag: Typed::new(CONTINUATION, 2),
            c: Typed::new(CONTINUATION, 5),
            d_tag: Typed::new(CONTINUATION, 3),
            d: Typed::new(CONTINUATION, 6),
        }
    };

    #[derive(Debug, Clone, PartialEq, Decode)]
    #[musli(packed)]
    struct Unpacked {
        count: Tag,
        a_tag: Typed<u8>,
        a: Typed<u8>,
        b_tag: Typed<u8>,
        b: Typed<u8>,
        c_tag: Typed<u8>,
        c: Typed<u8>,
        d_tag: Typed<u8>,
        d: Typed<u8>,
    }
}
