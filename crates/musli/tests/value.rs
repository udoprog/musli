use anyhow::Result;
use musli::alloc::Global;
use musli::json;
use musli::value::{self, Value};
use musli::{Decode, Encode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(default, skip_encoding_if = Option::is_none)]
    field: Option<u32>,
}

/// We want to assert that an option type can be decode from a plain unknown value.
#[test]
fn option() -> Result<()> {
    let value: Value<Global> = json::from_str(r#"{"field":null}"#)?;
    let st: Struct = value::decode_text(&value)?;
    assert_eq!(st.field, None);

    let value: Value<Global> = json::from_str(r#"{"field":42}"#)?;
    let st: Struct = value::decode_text(&value)?;
    assert_eq!(st.field, Some(42));

    let value: Value<Global> = json::from_str(r#"{}"#)?;
    let st: Struct = value::decode_text(&value)?;
    assert_eq!(st.field, None);
    Ok(())
}
