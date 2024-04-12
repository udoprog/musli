use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type", default_field = "name", default_variant = "name")]
pub enum InternallyTagged {
    Variant1 {
        string: String,
        number: u32,
    },
    #[musli(name = "variant2")]
    Variant2 {
        string: String,
    },
}

#[test]
fn internally_tagged() {
    tests::rt! {
        descriptive,
        InternallyTagged::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","string":"Hello","number":42}"#
    };

    tests::rt! {
        descriptive,
        InternallyTagged::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","string":"Hello"}"#
    };

    tests::rt! {
        descriptive,
        InternallyTagged::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","string":"\"\u0000"}"#
    };
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
    #[musli(name = "variant2")]
    Variant2 {
        string: String,
    },
}

#[test]
fn adjacently_tagged() {
    tests::rt! {
        descriptive,
        AdjacentlyTagged::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","content":{"string":"Hello","number":42}}"#
    };

    tests::rt! {
        descriptive,
        AdjacentlyTagged::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","content":{"string":"Hello"}}"#
    };

    tests::rt! {
        descriptive,
        AdjacentlyTagged::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","content":{"string":"\"\u0000"}}"#
    };
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(
    tag = "type",
    content = "content",
    default_field = "name",
    default_variant = "name"
)]
pub enum AdjacentlyTaggedWithSkip {
    Empty,
    Struct {
        string: String,
        #[musli(skip)]
        skipped: u32,
        number: u32,
    },
    #[musli(default_field = "index")]
    Tuple(String, #[musli(skip)] u32, u32),
}

#[test]
fn adjacently_tagged_with_skip() {
    tests::assert_decode_eq! {
        descriptive,
        AdjacentlyTaggedWithSkip::Empty,
        AdjacentlyTaggedWithSkip::Empty,
        json = r#"{"type":"Empty","content":{}}"#
    };

    tests::assert_decode_eq! {
        descriptive,
        AdjacentlyTaggedWithSkip::Struct {
            string: String::from("Hello World"),
            skipped: 42,
            number: 42,
        },
        AdjacentlyTaggedWithSkip::Struct {
            string: String::from("Hello World"),
            skipped: 0,
            number: 42,
        },
        json = r#"{"type":"Struct","content":{"string":"Hello World","number":42}}"#,
    };
}
