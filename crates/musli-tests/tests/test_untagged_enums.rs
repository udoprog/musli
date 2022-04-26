use musli::compat::Packed;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
pub enum UntaggedEnum1 {
    Variant1,
    Variant2,
}

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
pub enum UntaggedEnum2 {
    Variant1(String),
    Variant2(u32),
}

#[derive(Debug, PartialEq, Encode)]
#[musli(packed)]
pub enum UntaggedEnum3 {
    Variant1 { value: String },
    Variant2 { value: u32 },
}

#[derive(Debug, PartialEq, Decode)]
#[musli(packed)]
pub struct Empty;

/// Untagged enums may only implement `Encode`, and will be encoded according to
/// the exact specification of fields part of the variant.
#[test]
fn test_untagged_enums() -> Result<(), Box<dyn std::error::Error>> {
    let out = musli_tests::wire::to_vec(&UntaggedEnum1::Variant1).unwrap();
    let _: Empty = musli_tests::wire::decode(&out[..]).unwrap();

    let out = musli_tests::wire::to_vec(&UntaggedEnum1::Variant2).unwrap();
    let _: Empty = musli_tests::wire::decode(&out[..]).unwrap();

    let out = musli_tests::wire::to_vec(&UntaggedEnum2::Variant1(String::from("foo"))).unwrap();
    let Packed((value,)): Packed<(String,)> = musli_tests::wire::decode(&out[..]).unwrap();
    assert_eq!(value, "foo");

    let out = musli_tests::wire::to_vec(&UntaggedEnum2::Variant2(42)).unwrap();
    let Packed((value,)): Packed<(u32,)> = musli_tests::wire::decode(&out[..]).unwrap();
    assert_eq!(value, 42);

    let out = musli_tests::wire::to_vec(&UntaggedEnum3::Variant1 {
        value: String::from("foo"),
    })
    .unwrap();
    let Packed((value,)): Packed<(String,)> = musli_tests::wire::decode(&out[..]).unwrap();
    assert_eq!(value, "foo");

    let out = musli_tests::wire::to_vec(&UntaggedEnum3::Variant2 { value: 42 }).unwrap();
    let Packed((value,)): Packed<(u32,)> = musli_tests::wire::decode(&out[..]).unwrap();
    assert_eq!(value, 42);
    Ok(())
}
