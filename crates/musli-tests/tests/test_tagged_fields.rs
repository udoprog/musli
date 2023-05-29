use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructFrom {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct StructTo {
    #[musli(rename = 1)]
    number: u32,
    #[musli(rename = 0)]
    string: String,
}

#[test]
fn test_struct_renamed() {
    let from = musli_tests::rt!(StructFrom {
        string: String::from("a string"),
        number: 42,
    });

    let to = musli_tests::rt!(StructTo {
        number: 42,
        string: String::from("a string"),
    });

    let out = musli_tests::wire::to_vec(&from).expect("failed to encode");
    let value: StructTo = musli_tests::wire::decode(out.as_slice()).expect("failed to decode");
    assert_eq!(value, to);
}
