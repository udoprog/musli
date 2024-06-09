use rust_alloc::string::String;

use crate::wire::tag::{Kind, Tag};
use crate::wire::test::Typed;
use crate::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name_all = "name")]
pub struct Named {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name_all = "index")]
pub struct Indexed {
    string: String,
    number: u32,
}

#[test]
fn named_struct_unpack() {
    let out = crate::wire::to_vec(&Named {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = crate::storage::decode(out.as_slice()).expect("failed to decode");

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

    #[derive(Debug, PartialEq, Decode)]
    #[musli(crate, packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_name: Typed<6>,
        field1_value: Typed<3>,
        field2_name: Typed<6>,
        field2_value: Tag,
    }
}

#[test]
fn indexed_struct_unpack() {
    let out = crate::wire::to_vec(&Indexed {
        string: String::from("foo"),
        number: 42,
    })
    .expect("failed to encode");

    let unpacked: Unpacked = crate::storage::decode(out.as_slice()).expect("failed to decode");

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
    #[musli(crate, packed)]
    pub struct Unpacked {
        field_count: Tag,
        field1_index: Tag,
        field1_value: Typed<3>,
        field2_index: Tag,
        field2_value: Tag,
    }
}
