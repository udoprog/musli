use musli::{Decode, Encode};
use musli_wire::test::Typed;
use musli_wire::types::TypeTag;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(field = "name")]
pub struct Named {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(field = "index")]
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
            field_count: Typed::new(TypeTag::PairSequence, 2),
            field1_name: Typed::new(TypeTag::Prefixed, String::from("string")),
            field1_value: Typed::new(TypeTag::Prefixed, String::from("foo")),
            field2_name: Typed::new(TypeTag::Prefixed, String::from("number")),
            field2_value: Typed::new(TypeTag::Continuation, 42),
        }
    );

    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Typed<u8>,
        field1_name: Typed<String>,
        field1_value: Typed<String>,
        field2_name: Typed<String>,
        field2_value: Typed<u32>,
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
            field_count: Typed::new(TypeTag::PairSequence, 2),
            field1_index: Typed::new(TypeTag::Continuation, 0),
            field1_value: Typed::new(TypeTag::Prefixed, String::from("foo")),
            field2_index: Typed::new(TypeTag::Continuation, 1),
            field2_value: Typed::new(TypeTag::Continuation, 42),
        }
    );

    #[derive(Debug, PartialEq, Decode)]
    #[musli(packed)]
    pub struct Unpacked {
        field_count: Typed<u8>,
        field1_index: Typed<u8>,
        field1_value: Typed<String>,
        field2_index: Typed<u8>,
        field2_value: Typed<u32>,
    }
}
