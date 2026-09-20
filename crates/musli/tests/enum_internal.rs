use musli::{Decode, Encode};

#[test]
fn named() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag = "type", name_all = "snake_case")]
    pub enum Enum {
        #[musli(name_all = "snake_case")]
        OneField { string: String, number: u32 },
        #[musli(name_all = "snake_case")]
        TwoFields { string: String },
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::OneField {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"one_field","string":"Hello","number":42}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::TwoFields {
            string: String::from("Hello")
        },
        json = r#"{"type":"two_fields","string":"Hello"}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::TwoFields {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"two_fields","string":"\"\u0000"}"#
    };
}

#[test]
fn transparent() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(name_all = "snake_case")]
    pub struct Struct {
        string: String,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag = "type", name_all = "snake_case")]
    pub enum Enum {
        #[musli(transparent, name_all = "snake_case")]
        Tuple(Struct),
        #[musli(transparent, name_all = "snake_case")]
        Struct { st: Struct },
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Tuple(Struct {
            string: String::from("Hello")
        }),
        json = r#"{"type":"tuple","string":"Hello"}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Tuple(Struct {
            string: String::from("\"\u{0000}")
        }),
        json = r#"{"type":"tuple","string":"\"\u0000"}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Struct {
            st: Struct {
                string: String::from("Hello")
            }
        },
        json = r#"{"type":"struct","string":"Hello"}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Struct {
            st: Struct {
                string: String::from("\"\u{0000}")
            }
        },
        json = r#"{"type":"struct","string":"\"\u0000"}"#
    };
}

#[test]
fn indexed() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name(type = $ty), tag(value = 11, type = $ty))]
            pub enum Indexed {
                #[musli(name = 22)]
                Variant1 { variant1: u32 },
                #[musli(name = 33)]
                Variant2 { variant2: u32 },
            }

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                Indexed::Variant1 { variant1: 10 },
                json = r#"{"11":22,"variant1":10}"#
            };

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                Indexed::Variant2 { variant2: 20 },
                json = r#"{"11":33,"variant2":20}"#
            };

            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name(type = $ty), tag(value = 11, type = $ty))]
            pub enum IndexedBounds {
                #[musli(name = <$ty>::MAX)]
                Variant1 { variant1: u32 },
                #[musli(name = <$ty>::MIN)]
                Variant2 { variant2: u32 },
            }

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                IndexedBounds::Variant1 { variant1: 10 },
                json = format!(r#"{{"11":{},"variant1":10}}"#, <$ty>::MAX)
            };

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                IndexedBounds::Variant2 { variant2: 20 },
                json = format!(r#"{{"11":{},"variant2":20}}"#, <$ty>::MIN)
            };
        }};
    }

    test_case!(u8);
    test_case!(u16);
    test_case!(u32);
    test_case!(u64);
    test_case!(u128);
    test_case!(i8);
    test_case!(i16);
    test_case!(i32);
    test_case!(i64);
    test_case!(i128);
    test_case!(usize);
    test_case!(isize);
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(Text, tag = "type")]
enum JsonTarget {
    #[musli(Text, name = "port")]
    Port { id: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(Text, tag = "type")]
enum JsonRealtime {
    #[musli(Text, name = "rtkit")]
    Rtkit,
}

#[test]
fn json_internally_tagged_enums_decode_from_immutable_input() {
    let target = JsonTarget::Port { id: 1 };

    assert_eq!(
        musli::json::from_slice::<JsonTarget>(br#"{"type":"port","id":1}"#).unwrap(),
        target
    );
    assert_eq!(
        musli::json::from_str::<JsonTarget>(r#"{"type":"port","id":1}"#).unwrap(),
        target
    );
    assert_eq!(
        musli::json::from_slice::<JsonTarget>(br#"{"id":1,"type":"port"}"#).unwrap(),
        target
    );
    assert_eq!(
        musli::json::from_str::<JsonTarget>(r#"{"id":1,"type":"port"}"#).unwrap(),
        target
    );
    assert_eq!(
        musli::json::from_slice::<JsonRealtime>(br#"{"type":"rtkit"}"#).unwrap(),
        JsonRealtime::Rtkit
    );
    assert_eq!(
        musli::json::from_str::<JsonRealtime>(r#"{"type":"rtkit"}"#).unwrap(),
        JsonRealtime::Rtkit
    );
}

#[test]
fn json_mutable_byte_cursor_decodes_tag_first_data_and_advances() {
    let mut bytes = &br#"{"type":"port","id":1}{"type":"port","id":2} suffix"#[..];
    let first: JsonTarget = musli::json::decode(&mut bytes).unwrap();
    assert_eq!(bytes, br#"{"type":"port","id":2} suffix"#);
    let second: JsonTarget = musli::json::decode(&mut bytes).unwrap();
    assert_eq!(first, JsonTarget::Port { id: 1 });
    assert_eq!(second, JsonTarget::Port { id: 2 });
    assert_eq!(bytes, b" suffix");
}

#[test]
fn json_mutable_byte_cursor_decodes_tag_last_data() {
    let mut bytes = &br#"{"id":1,"type":"port"} suffix"#[..];
    let target: JsonTarget = musli::json::decode(&mut bytes).unwrap();
    assert_eq!(target, JsonTarget::Port { id: 1 });
    assert_eq!(bytes, b" suffix");
}

#[test]
fn json_mutable_byte_cursor_decodes_unit() {
    let mut bytes = &br#"{"type":"rtkit"} suffix"#[..];
    let realtime: JsonRealtime = musli::json::decode(&mut bytes).unwrap();
    assert_eq!(realtime, JsonRealtime::Rtkit);
    assert_eq!(bytes, b" suffix");
}

#[test]
fn json_mutable_string_cursor_decodes_tag_first_data_and_advances() {
    let mut string = r#"{"type":"port","id":1}{"type":"port","id":2} suffix"#;
    let first: JsonTarget = musli::json::decode(&mut string).unwrap();
    assert_eq!(string, r#"{"type":"port","id":2} suffix"#);
    let second: JsonTarget = musli::json::decode(&mut string).unwrap();
    assert_eq!(first, JsonTarget::Port { id: 1 });
    assert_eq!(second, JsonTarget::Port { id: 2 });
    assert_eq!(string, " suffix");
}

#[test]
fn json_mutable_string_cursor_decodes_tag_last_data() {
    let mut string = r#"{"id":1,"type":"port"} suffix"#;
    let target: JsonTarget = musli::json::decode(&mut string).unwrap();
    assert_eq!(target, JsonTarget::Port { id: 1 });
    assert_eq!(string, " suffix");
}

#[test]
fn json_mutable_string_cursor_decodes_unit() {
    let mut string = r#"{"type":"rtkit"} suffix"#;
    let realtime: JsonRealtime = musli::json::decode(&mut string).unwrap();
    assert_eq!(realtime, JsonRealtime::Rtkit);
    assert_eq!(string, " suffix");
}
