use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct {
    field: String,
}

#[derive(Debug, PartialEq, Encode)]
#[musli(encode_only, untagged)]
enum Enum {
    Variant1,
    #[musli(transparent)]
    Variant2(Struct),
    Variant3 {
        field: String,
    },
    Variant4(Struct),
}

#[derive(Debug, PartialEq, Decode)]
struct DecodeVariant1;

#[derive(Debug, PartialEq, Decode)]
#[musli(transparent)]
struct DecodeVariant2(Struct);

#[derive(Debug, PartialEq, Decode)]
struct DecodeVariant3 {
    field: String,
}

#[derive(Debug, PartialEq, Decode)]
struct DecodeVariant4(Struct);

#[test]
fn untagged_enums() {
    musli::macros::assert_decode_eq! {
        full,
        Enum::Variant1,
        DecodeVariant1,
        json = r#"{}"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        Enum::Variant2(Struct { field: String::from("Hello") }),
        DecodeVariant2(Struct { field: String::from("Hello") }),
        json = r#"{"field":"Hello"}"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        Enum::Variant3 { field: String::from("Hello") },
        DecodeVariant3{ field: String::from("Hello") },
        json = r#"{"field":"Hello"}"#,
    };

    musli::macros::assert_decode_eq! {
        full,
        Enum::Variant4(Struct { field: String::from("Hello") }),
        DecodeVariant4(Struct { field: String::from("Hello") }),
        json = r#"{"0":{"field":"Hello"}}"#,
    };
}
