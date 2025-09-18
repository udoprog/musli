use bstr::BStr;
use musli::{Decode, Encode};

macro_rules! integer_tag {
    ($(($ty:ty, $test:ident)),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                #[derive(Debug, PartialEq, Encode, Decode)]
                #[musli(tag(value = 10, type = $ty))]
                #[musli(content(value = 20, type = $ty))]
                pub enum Name {
                    Value,
                }

                #[derive(Debug, PartialEq, Encode, Decode)]
                #[musli(tag(value = 10, type = $ty, method = "sized"))]
                #[musli(content(value = 20, type = $ty, method = "sized"))]
                pub enum NameValue {
                    Value,
                }

                musli::macros::assert_roundtrip_eq! {
                    full,
                    Name::Value
                };

                musli::macros::assert_roundtrip_eq! {
                    full,
                    NameValue::Value
                };
            }
        )*
    };
}

integer_tag! {
    (u8, test_u8),
    (u16, test_u16),
    (u32, test_u32),
    (u64, test_u64),
    (u128, test_u128),
    (i8, test_i8),
    (i16, test_i16),
    (i32, test_i32),
    (i64, test_i64),
    (i128, test_i128),
    (usize, test_usize),
    (isize, test_isize),
}

#[test]
fn integer() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = 10), content(value = 20))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        full,
        Enum::Value
    };
}

#[test]
fn integer_value() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = 10, method = "sized"))]
    #[musli(content(value = 20, method = "sized"))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        full,
        Enum::Value
    };
}

#[test]
fn string() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag = "tag")]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Value
    };
}

#[test]
fn string_value() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = "tag"))]
    #[musli(content(value = "content"))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Value
    };
}

#[test]
fn string_value_type() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = "tag", type = str))]
    #[musli(content(value = "content", type = str))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Value
    };
}

#[test]
fn string_value_type_unsized() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = "tag", type = str, method = "unsized"))]
    #[musli(content(value = "content", type = str, method = "unsized"))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Enum::Value
    };
}

#[test]
#[ignore = "TODO: figure out why this is currently not supported"]
fn bytes_value() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = b"tag", format_with = BStr::new))]
    #[musli(content(value = b"content", format_with = BStr::new))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        full,
        Enum::Value
    };
}

#[test]
#[ignore = "TODO: figure out why this is currently not supported"]
fn bytes_value_type() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = b"tag", type = [u8], format_with = BStr::new))]
    #[musli(content(value = b"content", type = [u8], format_with = BStr::new))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        full,
        Enum::Value
    };
}

#[test]
#[ignore = "TODO: figure out why this is currently not supported"]
fn unsized_bytes() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(tag(value = b"tag", type = [u8], method = "unsized_bytes", format_with = BStr::new))]
    #[musli(content(value = b"content", type = [u8], method = "unsized_bytes", format_with = BStr::new))]
    pub enum Enum {
        Value,
    }

    musli::macros::assert_roundtrip_eq! {
        full,
        Enum::Value
    };
}
