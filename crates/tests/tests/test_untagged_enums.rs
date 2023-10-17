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
#[cfg(feature = "test")]
fn test_untagged_enums() -> Result<(), Box<dyn std::error::Error>> {
    use musli::compat::Packed;

    let out = tests::wire::to_vec(&UntaggedEnum1::Variant1).unwrap();
    let _: Empty = tests::wire::decode(out.as_slice()).unwrap();

    let out = tests::wire::to_vec(&UntaggedEnum1::Variant2).unwrap();
    let _: Empty = tests::wire::decode(out.as_slice()).unwrap();

    let out = tests::wire::to_vec(&UntaggedEnum2::Variant1(String::from("foo"))).unwrap();
    let Packed((value,)): Packed<(String,)> = tests::wire::decode(out.as_slice()).unwrap();
    assert_eq!(value, "foo");

    let out = tests::wire::to_vec(&UntaggedEnum2::Variant2(42)).unwrap();
    let Packed((value,)): Packed<(u32,)> = tests::wire::decode(out.as_slice()).unwrap();
    assert_eq!(value, 42);

    let out = tests::wire::to_vec(&UntaggedEnum3::Variant1 {
        value: String::from("foo"),
    })
    .unwrap();
    let Packed((value,)): Packed<(String,)> = tests::wire::decode(out.as_slice()).unwrap();
    assert_eq!(value, "foo");

    let out = tests::wire::to_vec(&UntaggedEnum3::Variant2 { value: 42 }).unwrap();
    let Packed((value,)): Packed<(u32,)> = tests::wire::decode(out.as_slice()).unwrap();
    assert_eq!(value, 42);
    Ok(())
}
