use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type", default_field = "name", default_variant = "name")]
pub enum InternallyTagged {
    Variant1 {
        string: String,
        number: u32,
    },
    #[musli(rename = "variant2")]
    Variant2 {
        string: String,
    },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(
    tag = "type",
    content = "content",
    default_field = "name",
    default_variant = "name"
)]
pub enum AdjacentlyTagged {
    Variant1 {
        string: String,
        number: u32,
    },
    #[musli(rename = "variant2")]
    Variant2 {
        string: String,
    },
}

#[test]
fn internally_tagged() {
    tests::rt_self! {
        InternallyTagged::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","string":"Hello","number":42}"#
    };

    tests::rt_self! {
        InternallyTagged::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","string":"Hello"}"#
    };

    tests::rt_self! {
        InternallyTagged::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","string":"\"\u0000"}"#
    };
}

#[test]
fn adjacently_tagged() {
    tests::rt! {
        AdjacentlyTagged::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","content":{"string":"Hello","number":42}}"#
    };

    tests::rt! {
        AdjacentlyTagged::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","content":{"string":"Hello"}}"#
    };

    tests::rt! {
        AdjacentlyTagged::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","content":{"string":"\"\u0000"}}"#
    };
}
