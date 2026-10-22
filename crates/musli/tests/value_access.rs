//! Tests for holding and reading a [`Value`]: the default allocator, the typed
//! accessors and decoding a `Value<Global>` with decoders using any allocator.

use anyhow::Result;
use musli::alloc::{ArrayBuffer, Disabled, Global, Slice};
use musli::context;
use musli::value::{self, Any, Value};
use musli::{Decode, Encode};

/// `Value` names `Value<Global>`.
#[test]
fn default_allocator() {
    let value: Value = Value::from(42u32);
    let global: Value<Global> = value;
    assert!(global.is_u32());
}

#[test]
fn accessors() -> Result<()> {
    let value = Value::<Global>::from(true);
    assert_eq!(value.as_bool(), Some(true));
    assert_eq!(value.as_u64(), None);
    assert_eq!(value.as_f64(), None);
    assert_eq!(value.as_str(), None);

    let value = Value::<Global>::from('a');
    assert_eq!(value.as_char(), Some('a'));
    assert_eq!(value.as_str(), None);
    assert_eq!(value.as_u64(), None);

    let value = Value::<Global>::try_from("hello")?;
    assert_eq!(value.as_str(), Some("hello"));
    assert_eq!(value.as_bytes(), None);
    assert_eq!(value.as_char(), None);
    assert_eq!(value.as_sequence(), None);

    let value = Value::<Global>::try_from(b"hello")?;
    assert_eq!(value.as_bytes(), Some(&b"hello"[..]));
    assert_eq!(value.as_str(), None);
    assert_eq!(value.as_sequence(), None);

    let value: Value = value::encode([1u32, 2, 3])?;
    let items = value.as_sequence().expect("sequence");
    assert_eq!(items.len(), 3);
    assert_eq!(items[2].as_u64(), Some(3));
    assert_eq!(value.as_map(), None);
    assert_eq!(value.as_bytes(), None);

    let value = Value::new_map_in(
        [
            (Value::try_from("a")?, Value::from(1u8)),
            (Value::try_from("a")?, Value::from(2u8)),
        ],
        Global::new(),
    )?;
    let entries = value.as_map().expect("map");
    // Duplicate keys are kept in order.
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0.as_str(), Some("a"));
    assert_eq!(entries[0].1.as_u64(), Some(1));
    assert_eq!(entries[1].1.as_u64(), Some(2));
    assert_eq!(value.as_sequence(), None);

    let value = Value::<Global>::empty();
    assert_eq!(value.as_bool(), None);
    assert_eq!(value.as_i64(), None);
    assert_eq!(value.as_f64(), None);
    assert_eq!(value.as_str(), None);
    assert_eq!(value.as_bytes(), None);
    assert_eq!(value.as_sequence(), None);
    assert_eq!(value.as_map(), None);

    // An optional value is not looked through.
    let value: Value = value::encode(Some(42u32))?;
    assert_eq!(value.as_u64(), None);
    Ok(())
}

#[test]
fn as_i64() {
    assert_eq!(
        Value::<Global>::from(i8::MIN).as_i64(),
        Some(i8::MIN as i64)
    );
    assert_eq!(
        Value::<Global>::from(i16::MIN).as_i64(),
        Some(i16::MIN as i64)
    );
    assert_eq!(
        Value::<Global>::from(i32::MIN).as_i64(),
        Some(i32::MIN as i64)
    );
    assert_eq!(Value::<Global>::from(i64::MIN).as_i64(), Some(i64::MIN));
    assert_eq!(Value::<Global>::from(i64::MAX).as_i64(), Some(i64::MAX));
    assert_eq!(Value::<Global>::from(-1isize).as_i64(), Some(-1));
    assert_eq!(
        Value::<Global>::from(u8::MAX).as_i64(),
        Some(u8::MAX as i64)
    );
    assert_eq!(
        Value::<Global>::from(u32::MAX).as_i64(),
        Some(u32::MAX as i64)
    );
    assert_eq!(Value::<Global>::from(42usize).as_i64(), Some(42));
    assert_eq!(
        Value::<Global>::from(i64::MAX as u64).as_i64(),
        Some(i64::MAX)
    );

    // Out of range.
    assert_eq!(Value::<Global>::from(i64::MAX as u64 + 1).as_i64(), None);
    assert_eq!(Value::<Global>::from(u64::MAX).as_i64(), None);
    assert_eq!(Value::<Global>::from(i64::MIN as i128 - 1).as_i64(), None);
    assert_eq!(Value::<Global>::from(i64::MAX as i128 + 1).as_i64(), None);
    assert_eq!(Value::<Global>::from(i128::MIN).as_i64(), None);
    assert_eq!(Value::<Global>::from(u128::MAX).as_i64(), None);
    assert_eq!(Value::<Global>::from(-1i128).as_i64(), Some(-1));
    assert_eq!(Value::<Global>::from(42u128).as_i64(), Some(42));

    // Floats are not integers, even without a fractional part.
    assert_eq!(Value::<Global>::from(42.0f32).as_i64(), None);
    assert_eq!(Value::<Global>::from(42.0f64).as_i64(), None);
}

#[test]
fn as_u64() {
    assert_eq!(
        Value::<Global>::from(u8::MAX).as_u64(),
        Some(u8::MAX as u64)
    );
    assert_eq!(Value::<Global>::from(u64::MAX).as_u64(), Some(u64::MAX));
    assert_eq!(
        Value::<Global>::from(usize::MAX).as_u64(),
        Some(usize::MAX as u64)
    );
    assert_eq!(Value::<Global>::from(0i8).as_u64(), Some(0));
    assert_eq!(
        Value::<Global>::from(i8::MAX).as_u64(),
        Some(i8::MAX as u64)
    );
    assert_eq!(
        Value::<Global>::from(i64::MAX).as_u64(),
        Some(i64::MAX as u64)
    );
    assert_eq!(
        Value::<Global>::from(u64::MAX as u128).as_u64(),
        Some(u64::MAX)
    );
    assert_eq!(
        Value::<Global>::from(u64::MAX as i128).as_u64(),
        Some(u64::MAX)
    );

    // Negative and out of range.
    assert_eq!(Value::<Global>::from(-1i8).as_u64(), None);
    assert_eq!(Value::<Global>::from(i64::MIN).as_u64(), None);
    assert_eq!(Value::<Global>::from(-1isize).as_u64(), None);
    assert_eq!(Value::<Global>::from(u64::MAX as u128 + 1).as_u64(), None);
    assert_eq!(Value::<Global>::from(u64::MAX as i128 + 1).as_u64(), None);
    assert_eq!(Value::<Global>::from(i128::MIN).as_u64(), None);

    // Floats are not integers, even without a fractional part.
    assert_eq!(Value::<Global>::from(42.0f64).as_u64(), None);
}

#[test]
fn as_f64() {
    assert_eq!(Value::<Global>::from(1.5f64).as_f64(), Some(1.5));
    assert_eq!(Value::<Global>::from(f64::MAX).as_f64(), Some(f64::MAX));

    // An f32 widens exactly.
    assert_eq!(Value::<Global>::from(0.1f32).as_f64(), Some(0.1f32 as f64));
    assert_eq!(
        Value::<Global>::from(f32::MIN).as_f64(),
        Some(f32::MIN as f64)
    );

    let nan = Value::<Global>::from(f64::NAN).as_f64().expect("number");
    assert!(nan.is_nan());

    // Integers widen, exactly up to 2^53.
    assert_eq!(Value::<Global>::from(-42i8).as_f64(), Some(-42.0));
    assert_eq!(Value::<Global>::from(42u16).as_f64(), Some(42.0));
    assert_eq!(
        Value::<Global>::from(1i64 << 53).as_f64(),
        Some(9007199254740992.0)
    );
    assert_eq!(
        Value::<Global>::from(-(1i64 << 53)).as_f64(),
        Some(-9007199254740992.0)
    );
    assert_eq!(
        Value::<Global>::from(i128::MIN).as_f64(),
        Some(i128::MIN as f64)
    );
    assert_eq!(
        Value::<Global>::from(u128::MAX).as_f64(),
        Some(u128::MAX as f64)
    );

    // Beyond 2^53 the nearest float is returned.
    assert_eq!(
        Value::<Global>::from((1u64 << 53) + 1).as_f64(),
        Some(9007199254740992.0)
    );

    assert_eq!(Value::<Global>::from(true).as_f64(), None);
    assert_eq!(Value::<Global>::try_from("1.5").unwrap().as_f64(), None);
}

/// Numbers decoded from JSON are read with the accessors.
#[test]
fn json_numbers() -> Result<()> {
    let value: Value =
        musli::json::from_str(r#"[1, -1, 1.5, 18446744073709551615, -9223372036854775808]"#)?;
    let items = value.as_sequence().expect("sequence");

    assert_eq!(items[0].as_u64(), Some(1));
    assert_eq!(items[0].as_i64(), Some(1));
    assert_eq!(items[0].as_f64(), Some(1.0));

    assert_eq!(items[1].as_u64(), None);
    assert_eq!(items[1].as_i64(), Some(-1));
    assert_eq!(items[1].as_f64(), Some(-1.0));

    assert_eq!(items[2].as_u64(), None);
    assert_eq!(items[2].as_i64(), None);
    assert_eq!(items[2].as_f64(), Some(1.5));

    assert_eq!(items[3].as_u64(), Some(u64::MAX));
    assert_eq!(items[3].as_i64(), None);

    assert_eq!(items[4].as_u64(), None);
    assert_eq!(items[4].as_i64(), Some(i64::MIN));
    Ok(())
}

/// Every kind of value is told apart by matching on `as_any`.
#[test]
fn as_any() -> Result<()> {
    assert!(matches!(Value::<Global>::empty().as_any(), Any::Empty));
    assert!(matches!(
        Value::<Global>::from(true).as_any(),
        Any::Bool(true)
    ));
    assert!(matches!(
        Value::<Global>::from('a').as_any(),
        Any::Char('a')
    ));

    let Any::Number(n) = Value::<Global>::i8(-42).as_any() else {
        panic!("expected a number");
    };
    assert!(!n.is_float());
    assert_eq!(n.as_i64(), Some(-42));
    assert_eq!(n.as_u64(), None);
    assert_eq!(n.as_f64(), Some(-42.0));

    let Any::Number(n) = Value::<Global>::u64(u64::MAX).as_any() else {
        panic!("expected a number");
    };
    assert!(!n.is_float());
    assert_eq!(n.as_i64(), None);
    assert_eq!(n.as_u64(), Some(u64::MAX));

    let Any::Number(n) = Value::<Global>::f64(42.0).as_any() else {
        panic!("expected a number");
    };
    assert!(n.is_float());
    assert_eq!(n.as_i64(), None);
    assert_eq!(n.as_u64(), None);
    assert_eq!(n.as_f64(), Some(42.0));

    let value = Value::<Global>::try_from(b"hello")?;
    assert!(matches!(value.as_any(), Any::Bytes(b"hello")));

    let value = Value::<Global>::try_from("hello")?;
    assert!(matches!(value.as_any(), Any::String("hello")));

    let value: Value = value::encode([1u32, 2])?;
    let Any::Sequence(items) = value.as_any() else {
        panic!("expected a sequence");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].as_u64(), Some(2));

    let value = Value::new_map_in([(Value::try_from("a")?, Value::from(1u8))], Global::new())?;
    let Any::Map(entries) = value.as_any() else {
        panic!("expected a map");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0.as_str(), Some("a"));
    assert_eq!(entries[0].1.as_u64(), Some(1));

    let value: Value = value::encode(Event::Payload(Value::from(7u8)))?;
    let Any::Variant(tag, payload) = value.as_any() else {
        panic!("expected a variant, got {value:?}");
    };
    assert_eq!(tag.as_u64(), Some(1));
    // Tuple fields are encoded as a map indexed by position.
    let Any::Map(fields) = payload.as_any() else {
        panic!("expected tuple fields, got {payload:?}");
    };
    assert_eq!(fields[0].0.as_u64(), Some(0));
    assert_eq!(fields[0].1.as_u64(), Some(7));

    let value: Value = value::encode(Some(42u32))?;
    let Any::Option(Some(inner)) = value.as_any() else {
        panic!("expected some, got {value:?}");
    };
    assert_eq!(inner.as_u64(), Some(42));

    let value: Value = value::encode(None::<u32>)?;
    assert!(matches!(value.as_any(), Any::Option(None)));

    // JSON floats are floats, while integers are not.
    let value: Value = musli::json::from_str(r#"[1, 1.5]"#)?;
    let items = value.as_sequence().expect("sequence");
    assert!(matches!(items[0].as_any(), Any::Number(n) if !n.is_float()));
    assert!(matches!(items[1].as_any(), Any::Number(n) if n.is_float()));
    Ok(())
}

/// A JSON number written with a fraction or an exponent is a float, even when
/// it is whole, so it roundtrips as written.
#[test]
fn json_whole_floats() -> Result<()> {
    let value: Value = musli::json::from_str(r#"[5, 5.0, -5.0, 1e3]"#)?;
    let items = value.as_sequence().expect("sequence");

    assert!(matches!(items[0].as_any(), Any::Number(n) if !n.is_float()));
    assert!(matches!(items[1].as_any(), Any::Number(n) if n.is_float()));
    assert!(matches!(items[2].as_any(), Any::Number(n) if n.is_float()));
    assert!(matches!(items[3].as_any(), Any::Number(n) if n.is_float()));

    assert_eq!(items[1].as_i64(), None);
    assert_eq!(items[1].as_f64(), Some(5.0));
    assert_eq!(items[3].as_f64(), Some(1000.0));

    assert_eq!(musli::json::to_string(&value)?, "[5,5.0,-5.0,1000.0]");

    // A typed integer is still read from a whole number written as a float.
    #[cfg(feature = "parse-full")]
    assert_eq!(musli::json::from_str::<u32>("5.0")?, 5);
    Ok(())
}

/// A type holding a `Value` derives `Decode` without `#[musli(global)]`.
#[derive(Debug, PartialEq, Encode, Decode)]
struct Request {
    method: std::string::String,
    params: Value,
    optional: Option<Value>,
    list: Vec<Value>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Event {
    Empty,
    Payload(Value),
    Named { value: Value },
}

fn request() -> Result<Request> {
    Ok(Request {
        method: "add".into(),
        params: value::encode((1u32, "two", [3.5f64]))?,
        optional: Some(Value::try_from("hello")?),
        list: vec![Value::from(1u8), Value::try_from(b"bytes")?],
    })
}

/// Roundtrip through a self-describing binary format, which preserves the
/// exact representation of every value.
#[test]
fn derived_roundtrip() -> Result<()> {
    fn roundtrip<T>(expected: T) -> Result<()>
    where
        T: std::fmt::Debug
            + PartialEq
            + Encode<musli::mode::Binary>
            + musli::de::DecodeOwned<musli::mode::Binary, Global>,
    {
        let bytes = musli::descriptive::to_vec(&expected)?;
        let actual: T = musli::descriptive::from_slice(&bytes)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    roundtrip(request()?)?;
    roundtrip(Event::Empty)?;
    roundtrip(Event::Payload(Value::from(42u32)))?;
    roundtrip(Event::Named {
        value: Value::try_from(b"bytes")?,
    })?;
    Ok(())
}

/// The decoder uses the default allocator, which is not `Global`.
#[test]
fn derived_default_allocator() -> Result<()> {
    let request: Request = musli::json::from_str(
        r#"{"method": "add", "params": [1, "two"], "optional": null, "list": [true, {"a": 1}]}"#,
    )?;

    assert_eq!(request.method, "add");
    let params = request.params.as_sequence().expect("sequence");
    assert_eq!(params[0].as_u64(), Some(1));
    assert_eq!(params[1].as_str(), Some("two"));
    assert_eq!(request.optional, None);
    assert_eq!(request.list[0].as_bool(), Some(true));
    let map = request.list[1].as_map().expect("map");
    assert_eq!(map[0].0.as_str(), Some("a"));
    assert_eq!(map[0].1.as_u64(), Some(1));
    Ok(())
}

/// The decoder allocates from a slice, while the decoded values are allocated
/// in the global allocator.
#[test]
fn derived_slice_allocator() -> Result<()> {
    let expected = request()?;

    let bytes = musli::descriptive::to_vec(&expected)?;
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);
    let actual: Request = musli::descriptive::Encoding::new().from_slice_with(&cx, &bytes)?;
    assert_eq!(actual, expected);

    let json = musli::json::to_string(&expected)?;
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);
    let actual: Request = musli::json::Encoding::new().from_str_with(&cx, &json)?;
    assert_eq!(actual.method, expected.method);
    assert_eq!(actual.params.as_sequence().map(<[_]>::len), Some(3));
    assert_eq!(actual.optional, expected.optional);
    Ok(())
}

/// Allocating the decoded value never touches the decoder's allocator, so a
/// decoder which cannot allocate still produces a `Value<Global>`.
#[test]
fn derived_disabled_allocator() -> Result<()> {
    let expected = Event::Payload(value::encode([1u32, 2, 3])?);
    let bytes = musli::descriptive::to_vec(&expected)?;

    let cx = context::new_in(Disabled::new());
    let actual: Event = musli::descriptive::Encoding::new().from_slice_with(&cx, &bytes)?;
    assert_eq!(actual, expected);
    Ok(())
}

/// A value can still be decoded into the allocator of the decoder.
#[test]
fn decode_in_decoder_allocator() -> Result<()> {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);

    let value: Value<&Slice<'_>> =
        musli::json::Encoding::new().from_str_with(&cx, r#"{"a": [1, "b"]}"#)?;
    let map = value.as_map().expect("map");
    assert_eq!(map[0].0.as_str(), Some("a"));
    let items = map[0].1.as_sequence().expect("sequence");
    assert_eq!(items[0].as_u64(), Some(1));
    assert_eq!(items[1].as_str(), Some("b"));
    Ok(())
}

/// Malformed input holding a value is rejected with an error.
#[test]
fn derived_malformed() -> Result<()> {
    for input in [
        r#"{"method": "add", "params": [1, "two""#,
        r#"{"method": "add", "params": {"a": }, "optional": null, "list": []}"#,
        r#"{"method": "add", "params": [1, 2], "optional": null, "list": [tru]}"#,
        r#"{"method": "add", "params": "unterminated, "optional": null, "list": []}"#,
    ] {
        assert!(
            musli::json::from_str::<Request>(input).is_err(),
            "{input} should not decode"
        );
    }

    let bytes = musli::descriptive::to_vec(&request()?)?;

    for len in 0..bytes.len() {
        assert!(
            musli::descriptive::from_slice::<Request>(&bytes[..len]).is_err(),
            "truncated to {len} bytes should not decode"
        );
    }

    let bytes = musli::descriptive::to_vec(&Event::Named {
        value: value::encode(("a", [1u32, 2]))?,
    })?;

    for len in 0..bytes.len() {
        assert!(
            musli::descriptive::from_slice::<Event>(&bytes[..len]).is_err(),
            "truncated to {len} bytes should not decode"
        );
    }

    Ok(())
}
