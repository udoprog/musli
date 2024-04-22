#![cfg(feature = "test")]

use musli::{Decode, Encode};

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
    musli::rt!(
        full,
        Named {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"string":"foo","number":42}"#,
    );

    musli::rt!(
        full,
        NamedByType {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"string":"foo","number":42}"#,
    );
}

#[test]
fn indexed_struct() {
    musli::rt!(
        full,
        Indexed {
            string: String::from("foo"),
            number: 42,
        },
        json = r#"{"0":"foo","1":42}"#,
    );
}
