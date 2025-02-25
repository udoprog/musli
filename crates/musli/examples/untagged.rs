use musli::{Decode, Encode};

#[derive(Encode)]
struct Struct {
    field: String,
}

#[derive(Encode)]
#[musli(untagged)]
enum Enum {
    Variant,
    #[musli(transparent)]
    Variant2(Struct),
    Variant3 {
        field: String,
    },
    Variant4(Struct),
}

fn main() {
    let string = musli::json::to_string(&Enum::Variant).unwrap();
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
