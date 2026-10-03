//! Deeply nested input must not overflow the stack.
//!
//! A stack overflow aborts the process, so each decode runs on a thread with a
//! fixed stack size. That makes the outcome deterministic: with unbounded
//! recursion these tests abort, with bounded recursion they return.

use musli::{Decode, json};

/// Far too small to recurse once per level of the inputs below.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Deep enough that recursing once per level overflows `STACK_SIZE`.
const DEEP: usize = 100_000;

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
