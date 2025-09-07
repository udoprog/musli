use musli::{Decode, Encode};

#[test]
fn named() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(name_all = "name", tag = "type", content = "content")]
    pub enum Named {
        #[musli(name_all = "name")]
        Variant1 { string: String, number: u32 },
        #[musli(name = "variant2", name_all = "name")]
        Variant2 { string: String },
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Named::Variant1 {
            string: String::from("Hello"),
            number: 42,
        },
        json = r#"{"type":"Variant1","content":{"string":"Hello","number":42}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Named::Variant2 {
            string: String::from("Hello")
        },
        json = r#"{"type":"variant2","content":{"string":"Hello"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Named::Variant2 {
            string: String::from("\"\u{0000}")
        },
        json = r#"{"type":"variant2","content":{"string":"\"\u0000"}}"#
    };
}

#[test]
fn transparent() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Struct {
        string: String,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag = "type", content = "data")]
    pub enum Enum {
        #[musli(transparent)]
        Tuple(Struct),
        #[musli(transparent)]
        Struct { st: Struct },
        #[musli(transparent)]
        StructSkip {
            #[musli(skip)]
            a: u32,
            st: Struct,
            #[musli(skip)]
            b: u32,
        },
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Tuple(Struct {
            string: String::from("Hello")
        }),
        json = r#"{"type":"Tuple","data":{"string":"Hello"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Tuple(Struct {
            string: String::from("\"\u{0000}")
        }),
        json = r#"{"type":"Tuple","data":{"string":"\"\u0000"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Struct {
            st: Struct {
                string: String::from("Hello")
            }
        },
        json = r#"{"type":"Struct","data":{"string":"Hello"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Struct {
            st: Struct {
                string: String::from("\"\u{0000}")
            }
        },
        json = r#"{"type":"Struct","data":{"string":"\"\u0000"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::StructSkip {
            a: 0,
            st: Struct {
                string: String::from("Hello")
            },
            b: 0,
        },
        json = r#"{"type":"StructSkip","data":{"string":"Hello"}}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::StructSkip {
            a: 0,
            st: Struct {
                string: String::from("\"\u{0000}")
            },
            b: 0,
        },
        json = r#"{"type":"StructSkip","data":{"string":"\"\u0000"}}"#
    };
}

#[test]
fn indexed() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name(type = $ty), tag(value = 11, type = $ty), content(value = 22, type = $ty))]
            pub enum Indexed {
                #[musli(name = 33, name_all = "name")]
                Variant1 { variant1: u32 },
                #[musli(name = 44, name_all = "name")]
                Variant2 { variant2: u32 },
            }

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                Indexed::Variant1 { variant1: 10 },
                json = r#"{"11":33,"22":{"variant1":10}}"#
            };

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                Indexed::Variant2 { variant2: 20 },
                json = r#"{"11":44,"22":{"variant2":20}}"#
            };

            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name(type = $ty), tag(value = 11, type = $ty), content(value = 22, type = $ty))]
            pub enum IndexedBounds {
                #[musli(name = <$ty>::MAX, name_all = "name")]
                Variant1 { variant1: u32 },
                #[musli(name = <$ty>::MIN, name_all = "name")]
                Variant2 { variant2: u32 },
            }

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                IndexedBounds::Variant1 { variant1: 10 },
                json = format!(r#"{{"11":{},"22":{{"variant1":10}}}}"#, <$ty>::MAX)
            };

            musli::macros::assert_roundtrip_eq! {
                descriptive,
                IndexedBounds::Variant2 { variant2: 20 },
                json = format!(r#"{{"11":{},"22":{{"variant2":20}}}}"#, <$ty>::MIN)
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
#[musli(tag = "type", content = "content", name_all = "name")]
pub enum NamedWithSkip {
    Empty,
    #[musli(name_all = "name")]
    Struct {
        string: String,
        #[musli(skip)]
        skipped: u32,
        number: u32,
    },
    #[musli(name_all = "index")]
    Tuple(String, #[musli(skip)] u32, u32),
}

#[test]
fn named_with_skip() {
    musli::macros::assert_decode_eq! {
        descriptive,
        NamedWithSkip::Empty,
        NamedWithSkip::Empty,
        json = r#"{"type":"Empty","content":{}}"#
    };

    musli::macros::assert_decode_eq! {
        descriptive,
        NamedWithSkip::Struct {
            string: String::from("Hello World"),
            skipped: 42,
            number: 42,
        },
        NamedWithSkip::Struct {
            string: String::from("Hello World"),
            skipped: 0,
            number: 42,
        },
        json = r#"{"type":"Struct","content":{"string":"Hello World","number":42}}"#,
    };
}

/// See https://github.com/udoprog/musli/issues/320
#[test]
fn issue_320() {
    #[derive(Debug, PartialEq, Decode, Encode)]
    #[musli(Text, name_all = "snake_case", tag = "type")]
    enum Criteria {
        Greeting,
        #[musli(Text, name(type = str))]
        IsHungry(#[musli(Text, name = "0")] u8),
        #[musli(Text, name(type = str))]
        IsTest(#[musli(Text, name = "0")] u8, #[musli(Text, name = "1")] u8),
    }

    #[derive(Debug, PartialEq, Decode, Encode)]
    struct MyRecord {
        crit: Criteria,
        text: Vec<String>,
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        MyRecord {
            crit: Criteria::Greeting,
            text: vec!["I am VERY hungry".to_string()],
        },
        json = r#"{"crit":{"type":"greeting"},"text":["I am VERY hungry"]}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Criteria::IsHungry(5),
        json = r#"{"type":"is_hungry","0":5}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        MyRecord {
            crit: Criteria::IsHungry(5),
            text: vec!["I am VERY hungry".to_string()],
        },
        json = r#"{"crit":{"type":"is_hungry","0":5},"text":["I am VERY hungry"]}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Criteria::IsTest(3, 2),
        json = r#"{"type":"is_test","0":3,"1":2}"#
    };

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        MyRecord {
            crit: Criteria::IsTest(3, 2),
            text: vec!["I am hungry".to_string()],
        },
        json = r#"{"crit":{"type":"is_test","0":3,"1":2},"text":["I am hungry"]}"#
    };
}
