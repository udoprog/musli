use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(
    tag = "type",
    default_field_name = "name",
    default_variant_name = "name"
)]
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

macro_rules! test {
    ($expr:expr $(, json = $json:expr)?) => {{
        let expected = $expr;
        let string = musli_json::to_string(&expected).unwrap();
        $(assert_eq!(string, $json);)*
        let output: InternallyTagged = musli_json::from_slice(string.as_bytes()).unwrap();
        assert_eq!(output, expected);
    }};
}

#[test]
fn test_internally_tagged() {
    test! {
        InternallyTagged::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = "{\"type\":\"Variant1\",\"string\":\"Hello\",\"number\":42}"
    };

    test! {
        InternallyTagged::Variant2 {
            string: String::from("Hello")
        },
        json = "{\"type\":\"variant2\",\"string\":\"Hello\"}"
    };

    test! {
        InternallyTagged::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = "{\"type\":\"variant2\",\"string\":\"\\\"\\u0000\"}"
    };
}
