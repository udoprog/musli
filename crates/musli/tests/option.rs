use musli::value;
use musli::{Decode, Encode};

use anyhow::Result;

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct {
    value: Option<Option<bool>>,
    inner: Option<Box<Struct>>,
}

#[test]
fn nested_option() -> Result<()> {
    let a = value::encode(Some(Some(true)))?;
    let b = value::encode(Some(None::<bool>))?;
    let c = value::encode(None::<Option<bool>>)?;

    assert_eq!(value::decode(&a), Ok(Some(Some(true))));
    assert_eq!(value::decode(&b), Ok(Some(None::<bool>)));
    assert_eq!(value::decode(&c), Ok(None::<Option<bool>>));

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Struct {
            value: Some(Some(true)),
            inner: Some(Box::new(Struct {
                value: Some(Some(true)),
                inner: None,
            })),
        },
        json = r#"{"value":true,"inner":{"value":true,"inner":null}}"#
    };

    Ok(())
}
