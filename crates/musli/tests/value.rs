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

/// Numbers are implicitly converted when decoded out of a value.
#[test]
fn number_coercion() -> Result<()> {
    let value = Value::<Global>::from(42u8);
    assert_eq!(value::decode::<u64>(&value)?, 42u64);
    assert_eq!(value::decode::<i32>(&value)?, 42i32);
    assert_eq!(value::decode::<f32>(&value)?, 42.0f32);

    let value = Value::<Global>::from(-1i64);
    assert_eq!(value::decode::<i8>(&value)?, -1i8);
    assert_eq!(value::decode::<f64>(&value)?, -1.0f64);
    assert!(value::decode::<u32>(&value).is_err());

    let value = Value::<Global>::from(3.5f32);
    assert_eq!(value::decode::<f64>(&value)?, 3.5f64);
    assert_eq!(value::decode::<u32>(&value)?, 3u32);

    let value = Value::<Global>::from(u128::MAX);
    assert_eq!(value::decode::<u128>(&value)?, u128::MAX);
    assert!(value::decode::<i128>(&value).is_err());

    let value = Value::<Global>::from(i128::MIN);
    assert_eq!(value::decode::<i128>(&value)?, i128::MIN);
    Ok(())
}

/// The exact number representation is preserved.
#[test]
fn number_representation() -> Result<()> {
    let value: Value<Global> = value::encode(-3i16)?;
    assert!(value.is_i16());
    assert_eq!(format!("{value:?}"), "-3");

    let value: Value<Global> = value::encode(1.5f32)?;
    assert!(value.is_f32());
    assert_eq!(format!("{value:?}"), "1.5");
    Ok(())
}

/// A `usize` and an `isize` use the platform specific representation.
#[test]
fn platform_sized_integers() -> Result<()> {
    let value: Value<Global> = value::encode(42usize)?;
    assert!(value.is_usize());
    #[cfg(target_pointer_width = "32")]
    assert_eq!(value, Value::from(42u32));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(value, Value::from(42u64));

    let value: Value<Global> = value::encode(-42isize)?;
    assert!(value.is_isize());
    #[cfg(target_pointer_width = "32")]
    assert_eq!(value, Value::from(-42i32));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(value, Value::from(-42i64));
    Ok(())
}
