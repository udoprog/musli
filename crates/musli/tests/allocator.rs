use anyhow::Result;
use musli::alloc::Global;
use musli::value::{self, Value};
use musli::{Allocator, Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct<A = Global>
where
    A: Allocator,
{
    value: Value<A>,
}

#[test]
fn with_allocator() -> Result<()> {
    assert_eq!(
        value::encode(Value::<Global>::empty())?,
        Value::<Global>::empty()
    );

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Struct::<Global> {
            value: Value::empty(),
        },
        json = r#"{"value":null}"#
    };

    Ok(())
}
