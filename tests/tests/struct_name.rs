#![cfg(feature = "test")]

use musli::{Decode, Encode};
use tests::wire::tag::{Kind, Tag};
use tests::wire::Typed;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "name")]
pub struct Named {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = str)]
pub struct NamedByType {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "index")]
pub struct Indexed {
    string: String,
    number: u32,
}

#[test]
fn named_struct() {
    tests::rt!(
        full,
        Named {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"string":"foo","number":42}"#,
    );

    tests::rt!(
        full,
        NamedByType {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"string":"foo","number":42}"#,
    );
}

#[test]
fn named_struct_unpack() {
    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_name: Typed<[u8; 6]>,
        field1_value: Typed<[u8; 3]>,
        field2_name: Typed<[u8; 6]>,
        field2_value: Tag,
    }

    let out = tests::wire::to_vec(&Named {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = tests::storage::decode(out.as_slice()).expect("failed to decode");

    assert_eq!(
        unpacked,
        Unpacked {
            field_count: Tag::new(Kind::Sequence, 4),
            field1_name: Typed::new(
                Tag::new(Kind::Prefix, 6),
                [b's', b't', b'r', b'i', b'n', b'g']
            ),
            field1_value: Typed::new(Tag::new(Kind::Prefix, 3), [b'f', b'o', b'o']),
            field2_name: Typed::new(
                Tag::new(Kind::Prefix, 6),
                [b'n', b'u', b'm', b'b', b'e', b'r']
            ),
            field2_value: Tag::new(Kind::Continuation, 42),
        }
    );
}

#[test]
fn indexed_struct() {
    tests::rt!(
        full,
        Indexed {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"0":"foo","1":42}"#,
    );
}

#[test]
fn indexed_struct_unpack() {
    let out = tests::wire::to_vec(&Indexed {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = tests::storage::decode(out.as_slice()).expect("failed to decode");

    assert_eq!(
        unpacked,
        Unpacked {
            field_count: Tag::new(Kind::Sequence, 4),
            field1_index: Tag::new(Kind::Continuation, 0),
            field1_value: Typed::new(Tag::new(Kind::Prefix, 3), [b'f', b'o', b'o']),
            field2_index: Tag::new(Kind::Continuation, 1),
            field2_value: Tag::new(Kind::Continuation, 42),
        }
    );

    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_index: Tag,
        field1_value: Typed<[u8; 3]>,
        field2_index: Tag,
        field2_value: Tag,
    }
}
