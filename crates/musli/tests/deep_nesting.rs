//! Deeply nested input must not overflow the stack.
//!
//! A stack overflow aborts the process, so each decode runs on a thread with a
//! fixed stack size. That makes the outcome deterministic: with unbounded
//! recursion these tests abort, with bounded recursion they return.

use musli::alloc::Global;

type Value = musli::value::Value<Global>;
use musli::{Decode, descriptive, json, sqlite_jsonb};

/// Large enough to decode a value nested up to the recursion limit in debug
/// builds, far too small to recurse once per level of the inputs below.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Deep enough that recursing once per level overflows `STACK_SIZE`.
const DEEP: usize = 100_000;

/// The nesting depth that the default context accepts.
const LIMIT: usize = 128;

fn with_stack<T>(f: impl FnOnce() -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(f)
        .expect("spawn thread")
        .join()
        .expect("thread panicked")
}

fn nested(open: &str, inner: &str, close: &str, depth: usize) -> String {
    let mut out = String::with_capacity((open.len() + close.len()) * depth + inner.len());

    for _ in 0..depth {
        out.push_str(open);
    }

    out.push_str(inner);

    for _ in 0..depth {
        out.push_str(close);
    }

    out
}

#[derive(Debug, PartialEq, Decode)]
struct Small {
    a: u32,
}

fn decode_small(input: String) -> Result<Small, String> {
    with_stack(move || json::from_slice::<Small>(input.as_bytes()).map_err(|e| e.to_string()))
}

#[test]
fn json_skip_deep_arrays() {
    let input = format!(r#"{{"a":1,"x":{}}}"#, nested("[", "", "]", DEEP));
    assert_eq!(decode_small(input), Ok(Small { a: 1 }));
}

#[test]
fn json_skip_deep_objects() {
    let input = format!(r#"{{"a":1,"x":{}}}"#, nested(r#"{"x":"#, "1", "}", DEEP));
    assert_eq!(decode_small(input), Ok(Small { a: 1 }));
}

#[test]
fn json_skip_deep_mixed() {
    let input = format!(
        r#"{{"x":{},"a":1}}"#,
        nested(r#"[1,"s",{"k":true,"x":"#, "null", "}]", DEEP)
    );
    assert_eq!(decode_small(input), Ok(Small { a: 1 }));
}

/// Skipping accepts and rejects the same inputs as decoding into a `Value`
/// does at shallow depths.
#[test]
fn json_skip_grammar() {
    let cases: &[(&str, bool)] = &[
        // Accepted.
        (r#"[]"#, true),
        (r#"{}"#, true),
        (r#"[1,"two",null,true,false,{"a":[1.5e3]}]"#, true),
        (r#"{"a":{"b":{"c":[]}},"d":-1}"#, true),
        (r#" [ 1 , [ ] , { } ] "#, true),
        (r#"[1,]"#, true),
        (r#"[1,,2]"#, true),
        (r#"[1 2]"#, true),
        (r#"{"a":1,}"#, true),
        (r#"{"a":1 "b":2}"#, true),
        (r#""s\"\\""#, true),
        // Rejected.
        (r#"["#, false),
        (r#"[[]"#, false),
        (r#"{"#, false),
        (r#"[}"#, false),
        (r#"{]"#, false),
        (r#"[1}"#, false),
        (r#"{"a":1]"#, false),
        (r#"[,]"#, false),
        (r#"[,1]"#, false),
        (r#"{,}"#, false),
        (r#"{"a"}"#, false),
        (r#"{"a" 1}"#, false),
        (r#"{"a":}"#, false),
        (r#"{1:2}"#, false),
        (r#"{"a":1,"b"}"#, false),
        (r#"[:]"#, false),
        (r#"[tru]"#, false),
        (r#"[nul]"#, false),
        (r#"["s]"#, false),
        (r#"]"#, false),
        (r#"}"#, false),
        (r#":"#, false),
        (r#","#, false),
    ];

    for &(value, ok) in cases {
        let input = format!(r#"{{"x":{value},"a":1}}"#);
        let skipped = json::from_slice::<Small>(input.as_bytes());
        assert_eq!(skipped.is_ok(), ok, "skipping {value}: {skipped:?}");

        if ok {
            assert_eq!(skipped.unwrap(), Small { a: 1 });
        }
    }
}

#[track_caller]
fn assert_limited(result: Result<(), String>) {
    let error = result.unwrap_err();
    assert!(error.contains("Recursion limit exceeded"), "{error}");
}

fn json_value(input: String) -> Result<(), String> {
    with_stack(move || {
        json::from_slice::<Value>(input.as_bytes())
            .map(drop)
            .map_err(|e| e.to_string())
    })
}

#[test]
fn json_value_deep_arrays() {
    assert_limited(json_value(nested("[", "", "]", DEEP)));
}

#[test]
fn json_value_deep_objects() {
    assert_limited(json_value(nested(r#"{"x":"#, "1", "}", DEEP)));
}

#[test]
fn json_value_limit() {
    assert_eq!(json_value(nested("[", "", "]", LIMIT)), Ok(()));
    assert_eq!(json_value(nested(r#"{"x":"#, "1", "}", LIMIT)), Ok(()));
    assert_eq!(
        json_value(nested(r#"[{"x":"#, "1", "}]", LIMIT / 2)),
        Ok(())
    );

    let error = json_value(nested("[", "", "]", LIMIT + 1)).unwrap_err();
    assert!(error.contains("Recursion limit"), "{error}");
    assert_limited(json_value(nested(r#"{"x":"#, "1", "}", LIMIT + 1)));
}

/// Values nested inside of other types are limited too.
#[test]
fn json_value_in_vec_deep() {
    let input = format!("[{}]", nested("[", "", "]", DEEP));

    let result = with_stack(move || {
        json::from_slice::<Vec<Value>>(input.as_bytes())
            .map(drop)
            .map_err(|e| e.to_string())
    });

    assert_limited(result);
}

fn descriptive_value(input: Vec<u8>) -> Result<(), String> {
    with_stack(move || {
        descriptive::from_slice::<Value>(&input)
            .map(drop)
            .map_err(|e| e.to_string())
    })
}

#[test]
fn descriptive_value_deep() {
    assert_limited(descriptive_value(vec![0x61; 1 << 20]));
}

#[test]
fn descriptive_value_deep_nested_sequences() {
    let mut value = Value::from(0u32);

    for _ in 0..DEEP.min(LIMIT * 4) {
        value = Value::from(vec![value]);
    }

    let bytes = descriptive::to_vec(&value).unwrap();
    assert_limited(descriptive_value(bytes));
}

#[test]
fn descriptive_value_limit() {
    let mut value = Value::from(0u32);

    for _ in 0..LIMIT {
        value = Value::from(vec![value]);
    }

    let bytes = descriptive::to_vec(&value).unwrap();
    assert_eq!(descriptive_value(bytes), Ok(()));

    let bytes = descriptive::to_vec(&Value::from(vec![value])).unwrap();
    assert_limited(descriptive_value(bytes));
}

/// Encode `depth` nested JSONB arrays around an empty array.
fn jsonb_nested_arrays(depth: usize) -> Vec<u8> {
    const ARRAY: u8 = 0x0b;

    fn header(size: usize) -> Vec<u8> {
        match size {
            0..=11 => vec![((size as u8) << 4) | ARRAY],
            12..=0xff => vec![0xc0 | ARRAY, size as u8],
            0x100..=0xffff => {
                let mut out = vec![0xd0 | ARRAY];
                out.extend_from_slice(&(size as u16).to_be_bytes());
                out
            }
            _ => {
                let mut out = vec![0xe0 | ARRAY];
                out.extend_from_slice(&(size as u32).to_be_bytes());
                out
            }
        }
    }

    let mut headers = Vec::with_capacity(depth);
    let mut size = 1;

    for _ in 0..depth {
        let h = header(size);
        size += h.len();
        headers.push(h);
    }

    let mut out = Vec::with_capacity(size);

    for h in headers.iter().rev() {
        out.extend_from_slice(h);
    }

    out.push(ARRAY);
    out
}

fn sqlite_jsonb_value(input: Vec<u8>) -> Result<(), String> {
    with_stack(move || {
        sqlite_jsonb::from_slice::<Value>(&input)
            .map(drop)
            .map_err(|e| e.to_string())
    })
}

#[test]
fn sqlite_jsonb_value_deep() {
    assert_limited(sqlite_jsonb_value(jsonb_nested_arrays(DEEP)));
}

#[test]
fn sqlite_jsonb_value_limit() {
    assert_eq!(sqlite_jsonb_value(jsonb_nested_arrays(LIMIT - 1)), Ok(()));
    assert_limited(sqlite_jsonb_value(jsonb_nested_arrays(LIMIT)));
}
