use musli::{Decode, Encode};

/// https://github.com/udoprog/musli/issues/28
#[test]
fn bug_28_skip_any() {
    #[derive(Debug, Encode, Decode)]
    #[musli(default_field_name = "name")]
    struct T {
        a: u32,
    }

    #[track_caller]
    fn test_case(input: &str) {
        assert_eq!(musli_json::from_str::<T>(input).unwrap().a, 42, "{input}");
    }

    test_case(r#"{"x":"1", "a":42}"#);
    test_case(r#"{"x":1, "a":42}"#);
    test_case(r#"{"x":true, "a":42}"#);
    test_case(r#"{"x":false, "a":42}"#);
    test_case(r#"{"x":null, "a":42}"#);
    test_case(r#"{"x":{"a": true, "b": false}, "a":42}"#);
    test_case(r#"{"x":["a", true, false], "a":42}"#);
}

/// https://github.com/udoprog/musli/issues/29
#[test]
fn bug_29_whitespace() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(default_field_name = "name")]
    struct T {
        a: u32,
        b: u32,
    }

    fn test_case(input: &str) {
        let value = musli_json::from_str::<T>(input).unwrap();
        assert_eq!(value.a, 42, "{input}");
        assert_eq!(value.b, 43, "{input}");
    }

    test_case("{\"a\": 42, \"b\": 43 }");
    test_case("{\"a\":\t42, \"b\":\t43\t}");
    test_case("{\"a\":\n42, \"b\":\n43\n}");
    test_case("{\"a\":\r42, \"b\":\r43\r}");
}
