//! Tests for holding a [`String<Global>`] or a [`Vec<T, Global>`] in a derived
//! type without `#[musli(global)]` or `#[musli(allocator = <type>)]`, and
//! decoding it with decoders using any allocator.

use anyhow::Result;
use musli::alloc::{self, ArrayBuffer, Disabled, FromAllocator, Global, Slice, String, Vec};
use musli::context;
use musli::{Allocator, Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Request {
    name: String<Global>,
    tags: Vec<String<Global>, Global>,
    numbers: Vec<u32, Global>,
    #[musli(bytes)]
    data: Vec<u8, Global>,
    nested: Option<Vec<Vec<u64, Global>, Global>>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Event {
    Empty,
    Name(String<Global>),
    Items { items: Vec<u64, Global> },
}

/// A container generic over its allocator can still hold global containers.
#[derive(Debug, PartialEq, Encode, Decode)]
struct Mixed<A>
where
    A: Allocator,
{
    local: String<A>,
    global: String<Global>,
}

fn string(value: &str) -> String<Global> {
    String::from(std::string::String::from(value))
}

fn vec<T>(values: impl IntoIterator<Item = T>) -> Vec<T, Global> {
    Vec::from(values.into_iter().collect::<std::vec::Vec<_>>())
}

fn request() -> Request {
    Request {
        name: string("add"),
        tags: vec([string("a"), string("long tag which is not inlined")]),
        numbers: vec([1, 2, u32::MAX]),
        data: vec(*b"\x00\x01\xff bytes"),
        nested: Some(vec([vec([1u64, 2]), vec([]), vec([u64::MAX])])),
    }
}

fn events() -> [Event; 3] {
    [
        Event::Empty,
        Event::Name(string("name")),
        Event::Items {
            items: vec([1, 2, 3]),
        },
    ]
}

macro_rules! roundtrip {
    ($format:ident, $value:expr, $ty:ty) => {{
        let expected = $value;

        // Default allocator.
        let bytes = musli::$format::to_vec(&expected)?;
        let actual: $ty = musli::$format::from_slice(&bytes)?;
        assert_eq!(actual, expected, "{} default", stringify!($format));

        // Slice allocator.
        let mut buf = ArrayBuffer::new();
        let alloc = Slice::new(&mut buf);
        let cx = context::new_in(&alloc);
        let actual: $ty = musli::$format::Encoding::new().from_slice_with(&cx, &bytes)?;
        assert_eq!(actual, expected, "{} slice", stringify!($format));
    }};
}

/// Roundtrip through every format with the default and a slice allocator.
#[test]
fn derived_roundtrip() -> Result<()> {
    roundtrip!(storage, request(), Request);
    roundtrip!(wire, request(), Request);
    roundtrip!(descriptive, request(), Request);
    roundtrip!(json, request(), Request);

    for event in events() {
        roundtrip!(storage, event, Event);
    }

    for event in events() {
        roundtrip!(wire, event, Event);
    }

    for event in events() {
        roundtrip!(descriptive, event, Event);
    }

    for event in events() {
        roundtrip!(json, event, Event);
    }

    Ok(())
}

/// Text decoding with the default allocator, which is not `Global`. JSON
/// decodes bytes into an owned buffer, which is handed over through
/// `visit_owned`.
#[test]
fn derived_default_allocator() -> Result<()> {
    let request: Request = musli::json::from_str(
        r#"{"name": "a\"b", "tags": ["x", "y\n"], "numbers": [1, 2], "data": [0, 255], "nested": null}"#,
    )?;

    assert_eq!(request.name.as_str(), "a\"b");
    assert_eq!(request.tags.len(), 2);
    assert_eq!(request.tags[1].as_str(), "y\n");
    assert_eq!(request.numbers.as_slice(), [1, 2]);
    assert_eq!(request.data.as_slice(), [0, 255]);
    assert_eq!(request.nested, None);
    Ok(())
}

/// Decoding with the slice allocator copies any owned buffer it hands over into
/// the global allocator, including JSON bytes and escaped strings.
#[test]
fn derived_slice_allocator() -> Result<()> {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);

    let request: Request = musli::json::Encoding::new().from_str_with(
        &cx,
        r#"{"name": "a\"b", "tags": [], "numbers": [], "data": [1, 2, 3], "nested": [[7]]}"#,
    )?;

    assert_eq!(request.name.as_str(), "a\"b");
    assert_eq!(request.data.as_slice(), [1, 2, 3]);
    assert_eq!(request.nested, Some(vec([vec([7])])));
    Ok(())
}

/// Allocating the decoded containers never touches the decoder's allocator, so
/// a decoder which cannot allocate still produces them.
#[test]
fn derived_disabled_allocator() -> Result<()> {
    let expected = request();

    let bytes = musli::storage::to_vec(&expected)?;
    let cx = context::new_in(Disabled::new());
    let actual: Request = musli::storage::Encoding::new().from_slice_with(&cx, &bytes)?;
    assert_eq!(actual, expected);

    let bytes = musli::descriptive::to_vec(&expected)?;
    let cx = context::new_in(Disabled::new());
    let actual: Request = musli::descriptive::Encoding::new().from_slice_with(&cx, &bytes)?;
    assert_eq!(actual, expected);

    let json = musli::json::to_string(&Event::Name(string("plain")))?;
    let cx = context::new_in(Disabled::new());
    let actual: Event = musli::json::Encoding::new().from_str_with(&cx, &json)?;
    assert_eq!(actual, Event::Name(string("plain")));
    Ok(())
}

/// A generic container decodes its local fields in the decoder's allocator and
/// its global fields in the global allocator.
#[test]
fn mixed_allocators() -> Result<()> {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);

    let mixed: Mixed<&Slice<'_>> = musli::json::Encoding::new()
        .from_str_with(&cx, r#"{"local": "in slice", "global": "in global"}"#)?;
    assert_eq!(mixed.local.as_str(), "in slice");
    assert_eq!(mixed.global.as_str(), "in global");

    let mixed: Mixed<Global> = musli::json::from_str(r#"{"local": "a", "global": "b"}"#)?;
    assert_eq!(mixed.local.as_str(), "a");
    assert_eq!(mixed.global.as_str(), "b");
    Ok(())
}

/// Containers can still be decoded into the allocator of the decoder.
#[test]
fn decode_in_decoder_allocator() -> Result<()> {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc);

    let value: Vec<String<&Slice<'_>>, &Slice<'_>> =
        musli::json::Encoding::new().from_str_with(&cx, r#"["a", "b\"c"]"#)?;
    assert_eq!(value.len(), 2);
    assert_eq!(value[1].as_str(), "b\"c");
    Ok(())
}

/// Malformed input is rejected with an error.
#[test]
fn derived_malformed() -> Result<()> {
    for input in [
        r#"{"name": "add", "tags": ["a""#,
        r#"{"name": 1, "tags": [], "numbers": [], "data": [], "nested": null}"#,
        r#"{"name": "a", "tags": [1], "numbers": [], "data": [], "nested": null}"#,
        r#"{"name": "a", "tags": [], "numbers": [-1], "data": [], "nested": null}"#,
        r#"{"name": "a", "tags": [], "numbers": [], "data": [256], "nested": null}"#,
        r#"{"name": "a", "tags": [], "numbers": [], "data": "x", "nested": null}"#,
        r#"{"name": "unterminated, "tags": [], "numbers": [], "data": [], "nested": null}"#,
        "{\"name\": \"\\ud800\", \"tags\": [], \"numbers\": [], \"data\": [], \"nested\": null}",
    ] {
        assert!(
            musli::json::from_str::<Request>(input).is_err(),
            "{input} should not decode"
        );

        let mut buf = ArrayBuffer::new();
        let alloc = Slice::new(&mut buf);
        let cx = context::new_in(&alloc);
        assert!(
            musli::json::Encoding::new()
                .from_str_with::<_, Request>(&cx, input)
                .is_err(),
            "{input} should not decode with a slice allocator"
        );
    }

    let expected = request();

    macro_rules! truncated {
        ($format:ident) => {
            let bytes = musli::$format::to_vec(&expected)?;

            for len in 0..bytes.len() {
                assert!(
                    musli::$format::from_slice::<Request>(&bytes[..len]).is_err(),
                    "{} truncated to {len} bytes should not decode",
                    stringify!($format)
                );
            }
        };
    }

    truncated!(storage);
    truncated!(wire);
    truncated!(descriptive);

    // Invalid UTF-8 in a string decoded from a binary format.
    let bytes = musli::wire::to_vec(&Event::Name(string("abc")))?;
    let at = bytes.windows(3).position(|w| w == b"abc").expect("string");
    let mut bytes = bytes;
    bytes[at] = 0xff;
    assert!(musli::wire::from_slice::<Event>(&bytes).is_err());
    Ok(())
}

/// An owned buffer allocated by a global-backed allocator is taken over by
/// `Global` without copying, while other allocators decline.
#[test]
fn from_vec_takes_over_global_allocations() {
    alloc::default(|alloc| {
        let mut vec = Vec::new_in(alloc);
        vec.extend_from_slice(b"hello").unwrap();
        let ptr = vec.as_ptr();

        let vec = <Global as FromAllocator<_>>::__from_vec(vec).expect("taken over");
        assert_eq!(vec.as_slice(), b"hello");
        assert_eq!(vec.as_ptr(), ptr);

        let mut units = Vec::new_in(alloc);
        units.push(()).unwrap();
        units.push(()).unwrap();
        let units = <Global as FromAllocator<_>>::__from_vec(units).expect("taken over");
        assert_eq!(units.len(), 2);
    });

    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let mut vec = Vec::new_in(&alloc);
    vec.extend_from_slice(b"hello").unwrap();
    let vec = <Global as FromAllocator<_>>::__from_vec(vec).expect_err("declined");
    assert_eq!(vec.as_slice(), b"hello");
}
