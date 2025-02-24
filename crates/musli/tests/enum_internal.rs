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
