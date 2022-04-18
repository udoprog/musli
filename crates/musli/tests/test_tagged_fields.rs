use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructFrom {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructTo {
    #[musli(tag = 1)]
    number: u32,
    #[musli(tag = 0)]
    string: String,
}

#[test]
fn test_struct_renamed() {
    let from = musli_wire::test::rt(StructFrom {
        string: String::from("a string"),
        number: 42,
    });

    let to = musli_wire::test::rt(StructTo {
        number: 42,
        string: String::from("a string"),
    });

    let out = musli_wire::to_vec(&from).expect("failed to encode");
    let value: StructTo = musli_wire::decode(&out[..]).expect("failed to decode");
    assert_eq!(value, to);
}
