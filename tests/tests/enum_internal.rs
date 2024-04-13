use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type", name_all = "name")]
pub enum Named {
    #[musli(name_all = "name")]
    Variant1 { string: String, number: u32 },
    #[musli(name = "variant2", name_all = "name")]
    Variant2 { string: String },
}

#[test]
fn named() {
    tests::rt! {
        descriptive,
        Named::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","string":"Hello","number":42}"#
    };

    tests::rt! {
        descriptive,
        Named::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","string":"Hello"}"#
    };

    tests::rt! {
        descriptive,
        Named::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","string":"\"\u0000"}"#
    };
}
