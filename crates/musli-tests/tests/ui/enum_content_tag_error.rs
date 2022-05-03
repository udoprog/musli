use musli::{Encode, Decode};

#[derive(Encode, Decode)]
#[musli(packed, tag = "type")]
enum PackedAndTag {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(packed, content = "content")]
enum PackedAndContent {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(packed, tag = "type", content = "content")]
enum PackedAndTagContent {
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

fn main() {
}
