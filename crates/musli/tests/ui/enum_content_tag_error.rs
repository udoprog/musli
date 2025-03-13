use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(untagged, tag = "type")]
enum UntaggedAndTag1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(untagged, tag(value = "type"))]
enum UntaggedAndTag2 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(untagged, content = "content")]
enum UntaggedAndContent1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(untagged, content(value = "content"))]
enum UntaggedAndContent2 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(untagged, tag = "type", content = "content")]
enum UntaggedAndTagContent1 {
    Variant1,
}

#[derive(Encode, Decode)]
#[musli(untagged, tag(value = "type"), content(value = "content"))]
enum UntaggedAndTagContent2 {
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
