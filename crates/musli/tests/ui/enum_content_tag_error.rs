use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(unpacked, tag = "type")]
enum UnpackedAndTag1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(unpacked, tag(value = "type"))]
enum UnpackedAndTag2 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(unpacked, content = "content")]
enum UnpackedAndContent1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(unpacked, content(value = "content"))]
enum UnpackedAndContent2 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(unpacked, tag = "type", content = "content")]
enum UnpackedAndTagContent1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(unpacked, tag(value = "type"), content(value = "content"))]
enum UnpackedAndTagContent2 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(transparent, tag = "type")]
enum TransparentAndTag {
    Variant1(String),
}

#[derive(Encode, Decode)]
#[musli(transparent, content = "content")]
enum TransparentAndContent {
    Variant1(String),
}

#[derive(Encode, Decode)]
#[musli(transparent, tag = "type", content = "content")]
enum TransparentAndTagContent {
    Variant1(String),
}

fn main() {}
