use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(tag = "type", default_field_tag = "name", default_variant_tag = "name")]
pub enum InternallyTagged {
    Variant1 { string: String, number: u32 },
}

#[test]
fn test_internally_tagged() {
    let string = musli_json::to_string(&InternallyTagged::Variant1 {
        string: String::from("Hello"),
        number: 42,
    })
    .unwrap();
    println!("{}", string);
}
