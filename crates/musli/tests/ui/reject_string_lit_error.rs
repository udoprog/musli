use musli::{Decode, Encode};

#[derive(Encode, Decode)]
struct Struct1 {
    #[musli(skip_encoding_if = "String::is_empty")]
    field: String,
}

#[derive(Encode, Decode)]
struct Struct2 {
    #[musli(with = "encode_with")]
    field: String,
}

#[derive(Encode, Decode)]
struct Struct3 {
    #[musli(encode_with = "encode_with")]
    field: String,
}

#[derive(Encode, Decode)]
struct Struct4 {
    #[musli(decode_with = "decode_with")]
    field: String,
}

#[derive(Encode, Decode)]
struct Struct5 {
    #[musli(serialize_with = "serialize_with")]
    field: String,
}

#[derive(Encode, Decode)]
struct Struct6 {
    #[musli(deserialize_with = "deserialize_with")]
    field: String,
}

fn main() {
}