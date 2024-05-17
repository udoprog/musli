use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

#[test]
fn version1_to_2() {
    musli::macros::assert_decode_eq! {
        upgrade_stable,
        Version2 {
            name: String::from("Aristotle"),
            age: Some(62),
        },
        Version1 {
            name: String::from("Aristotle"),
        },
    };

    musli::macros::assert_decode_eq! {
        full,
        Version1 {
            name: String::from("Aristotle"),
        },
        Version2 {
            name: String::from("Aristotle"),
            age: None,
        },
    };
}
