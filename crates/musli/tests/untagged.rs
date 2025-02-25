use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode)]
struct Struct {
    field: String,
}

#[derive(Debug, PartialEq, Encode)]
#[musli(encode_only, untagged)]
enum Enum {
    Variant,
    #[musli(transparent)]
    Variant2(Struct),
    Variant3 {
        field: String,
    },
    Variant4(Struct),
}

#[derive(Debug, PartialEq, Decode)]
#[musli(transparent)]
struct DecodeVariant1(String);

#[test]
fn untagged_enums() {
    musli::macros::assert_decode_eq! {
        text_mode,
        Enum::Variant,
        DecodeVariant1(String::from("Variant")),
        json = r#""Variant""#,
    };
}

/*
fn main() {
    let string = musli::json::to_string(&).unwrap();
    dbg!(&string);
    let string = musli::json::to_string(&Enum::Variant2(Struct {
        field: String::from("Hello"),
    }))
    .unwrap();
    dbg!(&string);
    let string = musli::json::to_string(&Enum::Variant3 {
        field: String::from("Hello"),
    })
    .unwrap();
    dbg!(&string);
    let string = musli::json::to_string(&Enum::Variant4(Struct {
        field: String::from("Hello"),
    }))
    .unwrap();
    dbg!(&string);
}
*/
