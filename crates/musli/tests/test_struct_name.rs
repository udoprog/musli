use musli::{Decode, Encode};
use musli_wire::test::Typed;
use musli_wire::types::{Kind, Tag, CONTINUATION};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_field_tag = "name")]
pub struct Named {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_field_tag = "index")]
pub struct Indexed {
    string: String,
    number: u32,
}

#[test]
fn struct_named_fields() {
    musli_wire::test::rt(Named {
        string: String::from("foo"),
        number: 42,
    });

    let out = musli_wire::to_vec(&Named {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = musli_storage::decode(&out[..]).expect("failed to decode");

    assert_eq!(
        unpacked,
        Unpacked {
            field_count: Tag::new(Kind::PairSequence, 2),
            field1_name: Typed::new(
                Tag::new(Kind::Prefixed, 6),
                [b's', b't', b'r', b'i', b'n', b'g']
            ),
            field1_value: Typed::new(Tag::new(Kind::Prefixed, 3), [b'f', b'o', b'o']),
            field2_name: Typed::new(
                Tag::new(Kind::Prefixed, 6),
                [b'n', b'u', b'm', b'b', b'e', b'r']
            ),
            field2_value: Typed::new(CONTINUATION, 42),
        }
    );

    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_name: Typed<[u8; 6]>,
        field1_value: Typed<[u8; 3]>,
        field2_name: Typed<[u8; 6]>,
        field2_value: Typed<u8>,
    }
}

#[test]
fn struct_indexed_fields() {
    musli_wire::test::rt(Indexed {
        string: String::from("foo"),
        number: 42,
    });

    let out = musli_wire::to_vec(&Indexed {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = musli_storage::decode(&out[..]).expect("failed to decode");

    assert_eq!(
        unpacked,
        Unpacked {
            field_count: Tag::new(Kind::PairSequence, 2),
            field1_index: Typed::new(CONTINUATION, 0),
            field1_value: Typed::new(Tag::new(Kind::Prefixed, 3), [b'f', b'o', b'o']),
            field2_index: Typed::new(CONTINUATION, 1),
            field2_value: Typed::new(CONTINUATION, 42),
        }
    );

    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_index: Typed<u8>,
        field1_value: Typed<[u8; 3]>,
        field2_index: Typed<u8>,
        field2_value: Typed<u32>,
    }
}
