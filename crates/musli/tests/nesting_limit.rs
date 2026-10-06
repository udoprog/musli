//! Decoding recursive user types is bounded by the nesting limit of the
//! context in every format.
//!
//! A stack overflow aborts the process, so each decode of deep input runs on a
//! thread with a fixed stack size. That makes the outcome deterministic: with
//! unbounded recursion these tests abort, with bounded recursion they return.
//!
//! Deep inputs are assembled from the bytes which one level of nesting adds to
//! an encoding, so no deeply nested value has to be encoded or dropped.

use musli::alloc::Global;
use musli::context::{self, DEFAULT_NESTING_LIMIT};
use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

/// Large enough to decode input nested up to the default limit in debug
/// builds, far too small to recurse once per level of `DEEP` input.
const STACK_SIZE: usize = 2 * 1024 * 1024;

/// Large enough to decode input nested `RAISED` levels deep in debug builds.
const LARGE_STACK_SIZE: usize = 256 * 1024 * 1024;

/// Deep enough that recursing once per level overflows `STACK_SIZE`.
const DEEP: usize = 100_000;

/// A nesting depth well above the default limit.
const RAISED: usize = 1000;

const LIMIT: usize = DEFAULT_NESTING_LIMIT;

const MESSAGE: &str = "Recursion limit exceeded";

/// Nested sequences, where every level of `Nest` is exactly one level of
/// nesting.
#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(transparent)]
struct Nest(Vec<Nest>);

#[derive(Debug, PartialEq, Encode, Decode)]
enum Tree {
    Leaf,
    Node(Vec<Tree>),
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct List {
    value: u32,
    next: Option<Box<List>>,
}

/// A type which nests through a value of type `Self`, `depth` times around a
/// base value.
trait Recursive: Sized {
    fn base() -> Self;

    fn wrap(self) -> Self;

    fn build(depth: usize) -> Self {
        let mut value = Self::base();

        for _ in 0..depth {
            value = value.wrap();
        }

        value
    }
}

impl Recursive for Nest {
    fn base() -> Self {
        Nest(Vec::new())
    }

    fn wrap(self) -> Self {
        Nest(vec![self])
    }
}

impl Recursive for Tree {
    fn base() -> Self {
        Tree::Leaf
    }

    fn wrap(self) -> Self {
        Tree::Node(vec![self])
    }
}

impl Recursive for List {
    fn base() -> Self {
        List {
            value: 7,
            next: None,
        }
    }

    fn wrap(self) -> Self {
        List {
            value: 7,
            next: Some(Box::new(self)),
        }
    }
}

fn with_stack<T>(size: usize, f: impl FnOnce() -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(size)
        .spawn(f)
        .expect("spawn thread")
        .join()
        .expect("thread panicked")
}

/// Assemble the encoding of `depth` levels of nesting around a base value,
/// given the encodings of 0, 1 and 2 levels.
///
/// This relies on one level of nesting adding a fixed prefix and suffix
/// around the encoding of the level below it, which is verified.
fn assemble(e0: &[u8], e1: &[u8], e2: &[u8], depth: usize) -> Vec<u8> {
    let (prefix, suffix) = (0..=e1.len().saturating_sub(e0.len()))
        .filter(|&at| e1[at..].starts_with(e0))
        .map(|at| (&e1[..at], &e1[at + e0.len()..]))
        .find(|(prefix, suffix)| [*prefix, e1, *suffix].concat() == e2)
        .expect("one level of nesting adds a fixed prefix and suffix");

    let mut out = Vec::with_capacity((prefix.len() + suffix.len()) * depth + e0.len());

    for _ in 0..depth {
        out.extend_from_slice(prefix);
    }

    out.extend_from_slice(e0);

    for _ in 0..depth {
        out.extend_from_slice(suffix);
    }

    out
}

#[track_caller]
fn assert_limited<T>(result: Result<T, String>)
where
    T: core::fmt::Debug,
{
    let error = result.unwrap_err();
    assert!(error.contains(MESSAGE), "{error}");
}

#[test]
fn default_limit() {
    assert_eq!(DEFAULT_NESTING_LIMIT, 128);
}

/// Build the encoding of `depth` levels of nesting.
macro_rules! input {
    // Every level adds a fixed prefix and suffix to the encoding.
    (assemble, $ty:ty, $depth:expr) => {{
        let e0 = format::to_vec(&<$ty>::build(0)).unwrap();
        let e1 = format::to_vec(&<$ty>::build(1)).unwrap();
        let e2 = format::to_vec(&<$ty>::build(2)).unwrap();
        assemble(&e0, &e1, &e2, $depth)
    }};

    // Containers are prefixed with the size of their contents, so the
    // headers of every level have to be computed.
    (assemble_jsonb, $ty:ty, $depth:expr) => {{
        let e0 = format::to_vec(&<$ty>::build(0)).unwrap();
        let e1 = format::to_vec(&<$ty>::build(1)).unwrap();
        let e2 = format::to_vec(&<$ty>::build(2)).unwrap();
        assemble_jsonb(&e0, &e1, &e2, $depth)
    }};
}

/// A JSONB container which one level of nesting wraps around the level
/// below it, with the bytes it contains before and after it.
struct JsonbFrame<'a> {
    kind: u8,
    before: &'a [u8],
    after: &'a [u8],
}

/// Decode the JSONB header at the start of `bytes`, returning the element
/// kind, the header length and the payload length.
fn jsonb_header(bytes: &[u8]) -> (u8, usize, usize) {
    let kind = bytes[0] & 0x0f;

    let (header, size) = match bytes[0] >> 4 {
        size @ 0..=11 => (1, size as usize),
        n => {
            let len = 1 << (n - 12);
            let mut size = 0usize;

            for &b in &bytes[1..1 + len] {
                size = (size << 8) | b as usize;
            }

            (1 + len, size)
        }
    };

    (kind, header, size)
}

/// Encode a JSONB header with the smallest representation of `size`, like
/// the encoder does.
fn jsonb_encode_header(out: &mut Vec<u8>, kind: u8, size: usize) {
    match size {
        0..=11 => out.push(((size as u8) << 4) | kind),
        12..=0xff => out.extend_from_slice(&[0xc0 | kind, size as u8]),
        0x100..=0xffff => {
            out.push(0xd0 | kind);
            out.extend_from_slice(&(size as u16).to_be_bytes());
        }
        _ => {
            out.push(0xe0 | kind);
            out.extend_from_slice(&(size as u32).to_be_bytes());
        }
    }
}

/// Find the containers from the root of `e1` down to the element at `at`.
fn jsonb_frames(e1: &[u8], at: usize, len: usize) -> Option<Vec<JsonbFrame<'_>>> {
    let mut frames = Vec::new();
    let mut start = 0;
    let mut end = e1.len();

    while start != at {
        let (kind, header, size) = jsonb_header(&e1[start..]);
        let payload = start + header;

        if payload + size != end {
            return None;
        }

        let mut child = payload;

        loop {
            if child >= end {
                return None;
            }

            let (_, child_header, child_size) = jsonb_header(&e1[child..]);
            let child_end = child + child_header + child_size;

            if (child..child_end).contains(&at) {
                frames.push(JsonbFrame {
                    kind,
                    before: &e1[payload..child],
                    after: &e1[child_end..end],
                });

                start = child;
                end = child_end;
                break;
            }

            child = child_end;
        }
    }

    (end == at + len).then_some(frames)
}

/// Like [`assemble`], but for JSONB where every level of nesting is wrapped
/// in containers whose headers depend on the size of their contents.
fn assemble_jsonb(e0: &[u8], e1: &[u8], e2: &[u8], depth: usize) -> Vec<u8> {
    fn build(e0: &[u8], frames: &[JsonbFrame<'_>], depth: usize) -> Vec<u8> {
        // Compute the headers from the innermost level outwards.
        let mut headers = Vec::with_capacity(depth * frames.len());
        let mut len = e0.len();

        for _ in 0..depth {
            for frame in frames.iter().rev() {
                let size = frame.before.len() + len + frame.after.len();
                let mut header = Vec::new();
                jsonb_encode_header(&mut header, frame.kind, size);
                len = header.len() + size;
                headers.push(header);
            }
        }

        let mut out = Vec::with_capacity(len);
        let mut headers = headers.iter().rev();

        for _ in 0..depth {
            for frame in frames {
                out.extend_from_slice(headers.next().unwrap());
                out.extend_from_slice(frame.before);
            }
        }

        out.extend_from_slice(e0);

        for _ in 0..depth {
            for frame in frames.iter().rev() {
                out.extend_from_slice(frame.after);
            }
        }

        out
    }

    let frames = (0..=e1.len().saturating_sub(e0.len()))
        .filter(|&at| e1[at..].starts_with(e0))
        .filter_map(|at| jsonb_frames(e1, at, e0.len()))
        .find(|frames| build(e0, frames, 1) == e1 && build(e0, frames, 2) == e2)
        .expect("one level of nesting adds fixed containers");

    build(e0, &frames, depth)
}

macro_rules! byte_formats {
    ($($module:ident, $mode:ty, $input:ident;)*) => {$(
        mod $module {
            use super::*;

            use musli::$module as format;

            fn input<T>(depth: usize) -> Vec<u8>
            where
                T: Recursive + Encode<$mode>,
            {
                input!($input, T, depth)
            }

            /// Decode with the default context.
            fn decode<T>(stack: usize, input: Vec<u8>) -> Result<(), String>
            where
                T: 'static + for<'de> Decode<'de, $mode, Global>,
            {
                with_stack(stack, move || {
                    format::from_slice::<T>(&input)
                        .map(std::mem::forget)
                        .map_err(|e| e.to_string())
                })
            }

            /// Decode with a context configured by `limit`, where `None`
            /// disables the limit.
            fn decode_with<T>(stack: usize, limit: Option<usize>, input: Vec<u8>) -> bool
            where
                T: 'static + for<'de> Decode<'de, $mode, Global>,
            {
                with_stack(stack, move || {
                    let cx = context::new();

                    let cx = match limit {
                        Some(limit) => cx.with_nesting_limit(limit),
                        None => cx.without_nesting_limit(),
                    };

                    format::Encoding::new()
                        .from_slice_with::<_, T>(&cx, &input)
                        .map(std::mem::forget)
                        .is_ok()
                })
            }

            /// The assembled input is the same as encoding the value.
            #[test]
            fn input_matches_encoding() {
                for depth in [0, 1, 2, 3, 10, 50, 100] {
                    assert_eq!(input::<Nest>(depth), format::to_vec(&Nest::build(depth)).unwrap());
                    assert_eq!(input::<Tree>(depth), format::to_vec(&Tree::build(depth)).unwrap());
                    assert_eq!(input::<List>(depth), format::to_vec(&List::build(depth)).unwrap());
                }
            }

            #[test]
            fn round_trip() {
                for depth in 0..4 {
                    let bytes = input::<Nest>(depth);
                    assert_eq!(format::from_slice::<Nest>(&bytes).unwrap(), Nest::build(depth));
                    let bytes = input::<Tree>(depth);
                    assert_eq!(format::from_slice::<Tree>(&bytes).unwrap(), Tree::build(depth));
                    let bytes = input::<List>(depth);
                    assert_eq!(format::from_slice::<List>(&bytes).unwrap(), List::build(depth));
                }
            }

            #[test]
            fn nest_deep() {
                assert_limited(decode::<Nest>(STACK_SIZE, input::<Nest>(DEEP)));
            }

            #[test]
            fn tree_deep() {
                assert_limited(decode::<Tree>(STACK_SIZE, input::<Tree>(DEEP)));
            }

            #[test]
            fn list_deep() {
                assert_limited(decode::<List>(STACK_SIZE, input::<List>(DEEP)));
            }

            /// Every `Nest` is one level of nesting, so wrapping the base
            /// value `LIMIT - 1` times is exactly `LIMIT` levels.
            #[test]
            fn nest_limit() {
                assert_eq!(decode::<Nest>(STACK_SIZE, input::<Nest>(LIMIT - 1)), Ok(()));
                assert_limited(decode::<Nest>(STACK_SIZE, input::<Nest>(LIMIT)));
            }

            /// A node is a variant, the fields of its tuple and a sequence,
            /// so it is three levels of nesting. The leaf is a variant and its
            /// empty fields, so two levels. That makes 42 nodes exactly 128
            /// levels.
            #[test]
            fn tree_limit() {
                assert_eq!(3 * 42 + 2, LIMIT);
                assert_eq!(decode::<Tree>(STACK_SIZE, input::<Tree>(42)), Ok(()));
                assert_limited(decode::<Tree>(STACK_SIZE, input::<Tree>(43)));
            }

            /// Each node except the last is a struct with a present optional
            /// value, so it is two levels of nesting. The last node is one
            /// level.
            #[test]
            fn list_limit() {
                assert_eq!(decode::<List>(STACK_SIZE, input::<List>((LIMIT - 1) / 2)), Ok(()));
                assert_limited(decode::<List>(STACK_SIZE, input::<List>((LIMIT - 1) / 2 + 1)));
            }

            #[test]
            fn custom_limit() {
                assert!(decode_with::<Nest>(STACK_SIZE, Some(10), input::<Nest>(9)));
                assert!(!decode_with::<Nest>(STACK_SIZE, Some(10), input::<Nest>(10)));
                assert!(decode_with::<Nest>(STACK_SIZE, Some(1), input::<Nest>(0)));
                assert!(!decode_with::<Nest>(STACK_SIZE, Some(0), input::<Nest>(0)));
            }

            #[test]
            fn raised_limit() {
                let bytes = input::<Nest>(RAISED - 1);
                assert!(decode_with::<Nest>(LARGE_STACK_SIZE, Some(RAISED), bytes.clone()));
                assert!(!decode_with::<Nest>(LARGE_STACK_SIZE, Some(RAISED - 1), bytes.clone()));
                assert!(decode_with::<Nest>(LARGE_STACK_SIZE, None, bytes));

                let bytes = input::<List>(RAISED);
                assert!(decode_with::<List>(LARGE_STACK_SIZE, Some(2 * RAISED + 1), bytes.clone()));
                assert!(!decode_with::<List>(LARGE_STACK_SIZE, Some(2 * RAISED), bytes.clone()));
                assert!(decode_with::<List>(LARGE_STACK_SIZE, None, bytes));

                let bytes = input::<Tree>(RAISED);
                assert!(decode_with::<Tree>(LARGE_STACK_SIZE, Some(3 * RAISED + 2), bytes.clone()));
                assert!(!decode_with::<Tree>(LARGE_STACK_SIZE, Some(3 * RAISED + 1), bytes.clone()));
                assert!(decode_with::<Tree>(LARGE_STACK_SIZE, None, bytes));
            }

            /// A context which is reused after hitting the limit starts from
            /// the top again.
            #[test]
            fn reuse_after_error() {
                let deep = input::<Nest>(LIMIT);
                let ok = input::<Nest>(LIMIT - 1);

                let result = with_stack(STACK_SIZE, move || {
                    let cx = context::new();
                    let encoding = format::Encoding::new();
                    let first = encoding.from_slice_with::<_, Nest>(&cx, &deep).is_ok();
                    let second = encoding.from_slice_with::<_, Nest>(&cx, &ok).is_ok();
                    (first, second)
                });

                assert_eq!(result, (false, true));
            }
        }
    )*};
}

byte_formats! {
    storage, Binary, assemble;
    packed, Binary, assemble;
    wire, Binary, assemble;
    descriptive, Binary, assemble;
    json, Text, assemble;
    sqlite_jsonb, Text, assemble_jsonb;
}

/// Decoding from a [`Value`] is limited too.
///
/// [`Value`]: musli::value::Value
mod value {
    use super::*;

    use musli::value::{self, Value};

    fn nest(depth: usize) -> Value<Global> {
        let mut out = Value::from(Vec::new());

        for _ in 0..depth {
            out = Value::from(vec![out]);
        }

        out
    }

    fn decode(stack: usize, limit: Option<usize>, depth: usize) -> Result<(), String> {
        with_stack(stack, move || {
            let input = nest(depth);

            let cx = match limit {
                Some(limit) => context::new().with_trace().with_nesting_limit(limit),
                None => context::new().with_trace().without_nesting_limit(),
            };

            let result = value::decode_with::<_, Nest>(&cx, &input)
                .map(std::mem::forget)
                .map_err(|_| cx.report().to_string());

            // Dropping deeply nested input recurses too.
            std::mem::forget(input);
            result
        })
    }

    #[test]
    fn round_trip() {
        for depth in 0..4 {
            let input = value::encode(Nest::build(depth)).unwrap();
            assert_eq!(value::decode::<Nest>(&input).unwrap(), Nest::build(depth));
        }

        assert_eq!(value::encode(Nest::build(3)).unwrap(), nest(3));
    }

    #[test]
    fn nest_deep() {
        assert_limited(decode(STACK_SIZE, Some(LIMIT), DEEP));
    }

    #[test]
    fn nest_limit() {
        let input = nest(LIMIT - 1);
        assert_eq!(
            value::decode::<Nest>(&input).unwrap(),
            Nest::build(LIMIT - 1)
        );
        assert_eq!(decode(STACK_SIZE, Some(LIMIT), LIMIT - 1), Ok(()));
        assert_limited(decode(STACK_SIZE, Some(LIMIT), LIMIT));

        let error = value::decode::<Nest>(&nest(LIMIT)).unwrap_err();
        assert!(error.to_string().contains(MESSAGE), "{error}");
    }

    #[test]
    fn raised_limit() {
        assert_eq!(decode(LARGE_STACK_SIZE, Some(RAISED), RAISED - 1), Ok(()));
        assert_limited(decode(LARGE_STACK_SIZE, Some(RAISED - 1), RAISED - 1));
        assert_eq!(decode(LARGE_STACK_SIZE, None, RAISED - 1), Ok(()));
    }
}

/// Optional values decoded through the serde compatibility layer count as a
/// level of nesting.
mod serde_compat {
    use super::*;

    use musli::json;

    #[derive(Debug, PartialEq, ::serde::Deserialize)]
    struct SerdeList {
        next: Option<Box<SerdeList>>,
    }

    #[derive(Debug, PartialEq, Decode)]
    #[musli(transparent)]
    struct Wrapper(#[musli(with = musli::serde)] SerdeList);

    /// Each node is an object with a present optional value, so two levels
    /// of nesting, around an object with an absent one.
    fn input(depth: usize) -> String {
        let mut out = String::new();

        for _ in 0..depth {
            out.push_str(r#"{"next":"#);
        }

        out.push_str(r#"{"next":null}"#);

        for _ in 0..depth {
            out.push('}');
        }

        out
    }

    fn decode(input: String) -> Result<(), String> {
        with_stack(STACK_SIZE, move || {
            json::from_str::<Wrapper>(&input)
                .map(std::mem::forget)
                .map_err(|e| e.to_string())
        })
    }

    #[test]
    fn list_deep() {
        assert_limited(decode(input(DEEP)));
    }

    #[test]
    fn list_limit() {
        assert_eq!(decode(input((LIMIT - 1) / 2)), Ok(()));
        assert_limited(decode(input((LIMIT - 1) / 2 + 1)));
    }
}
