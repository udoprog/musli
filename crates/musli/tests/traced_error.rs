//! Errors returned through a context which traces carry the path to the value
//! being decoded and the position in the input, both in their `Display` and
//! through accessors, while the plain entry points stay untraced.

use core::error::Error as _;
use core::fmt;
use core::mem::size_of;

use musli::alloc::{ArrayBuffer, Global, Slice};
use musli::context::{self, ErrorMarker};
use musli::{Context, Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct PeopleFrom {
    people: Vec<PersonFrom>,
}

#[derive(Debug, Encode, Decode)]
struct PersonFrom {
    name: String,
    age: u32,
}

#[derive(Debug, Decode)]
struct PeopleTo {
    #[allow(unused)]
    people: Vec<PersonTo>,
}

#[derive(Debug, Decode)]
struct PersonTo {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    age: String,
}

fn people() -> PeopleFrom {
    PeopleFrom {
        people: vec![
            PersonFrom {
                name: "Aristotle".to_string(),
                age: 61,
            },
            PersonFrom {
                name: "Plato".to_string(),
                age: 80,
            },
        ],
    }
}

const JSON_MISMATCH: &str =
    r#"{"people": [{"name": "Aristotle", "age": "61"}, {"name": "Plato", "age": 80}]}"#;

#[test]
fn json_traced_display() {
    let error = musli::json::from_str_traced::<PeopleTo>(JSON_MISMATCH).unwrap_err();

    assert_eq!(
        error.to_string(),
        ".people[1].age: Expected string, found <number> (at bytes 73-74)"
    );
    assert_eq!(error.path(), Some(".people[1].age"));
    assert_eq!(error.range(), Some(73..74));
}

#[test]
fn json_traced_methods() {
    const ENCODING: musli::json::Encoding = musli::json::Encoding::new();

    let expected = ".people[1].age: Expected string, found <number> (at bytes 73-74)";

    let error = ENCODING
        .from_str_traced::<PeopleTo>(JSON_MISMATCH)
        .unwrap_err();
    assert_eq!(error.to_string(), expected);

    let error = ENCODING
        .from_slice_traced::<PeopleTo>(JSON_MISMATCH.as_bytes())
        .unwrap_err();
    assert_eq!(error.to_string(), expected);

    let error = musli::json::from_slice_traced::<PeopleTo>(JSON_MISMATCH.as_bytes()).unwrap_err();
    assert_eq!(error.to_string(), expected);

    let error = ENCODING
        .decode_traced::<_, PeopleTo>(JSON_MISMATCH.as_bytes())
        .unwrap_err();
    assert_eq!(error.to_string(), expected);
}

/// The error agrees with what the context reports through its side channel.
#[test]
fn json_traced_matches_report() {
    const ENCODING: musli::json::Encoding = musli::json::Encoding::new();

    let cx = context::new().with_trace().with_error();
    let error: musli::json::Error = ENCODING
        .from_str_with::<_, PeopleTo>(&cx, JSON_MISMATCH)
        .unwrap_err();
    assert_eq!(cx.report().to_string(), format!("{error}\n"));
}

/// Malformed JSON is reported at the position of the syntax error, inside the
/// value being decoded.
#[test]
fn json_traced_malformed() {
    let error = musli::json::from_str_traced::<PeopleTo>(
        r#"{"people": [{"name": "Aristotle", "age" "61"}]}"#,
    )
    .unwrap_err();
    assert_eq!(error.path(), Some(".people[0].age"));
    assert_eq!(
        error.to_string(),
        ".people[0].age: Expected colon `:`, was `\"` (at byte 40)"
    );

    let error =
        musli::json::from_str_traced::<PeopleTo>(r#"{"people": [{"name": "Aristot"#).unwrap_err();
    assert_eq!(error.path(), Some(".people[0].name"));
    assert_eq!(
        error.to_string(),
        ".people[0].name: End of input (at byte 29)"
    );

    // Trailing input is raised outside of any field.
    let error = musli::json::from_str_traced::<PeopleTo>(r#"{"people": []} x"#).unwrap_err();
    assert_eq!(error.path(), Some(""));
    assert_eq!(error.to_string(), "Trailing input (at byte 15)");
}

/// The plain entry points do not trace, and keep only the message.
#[test]
fn json_plain_untraced() {
    let error = musli::json::from_str::<PeopleTo>(JSON_MISMATCH).unwrap_err();
    assert_eq!(error.to_string(), "Expected string, found <number>");
    assert_eq!(error.path(), None);
    assert_eq!(error.range(), None);
}

/// A binary format reports the path and the position the truncated input ran
/// out at.
#[test]
fn storage_traced_truncated() {
    let bytes = musli::storage::to_vec(&people()).unwrap();

    let error =
        musli::storage::from_slice_traced::<PeopleFrom>(&bytes[..bytes.len() - 3]).unwrap_err();
    assert_eq!(
        error.to_string(),
        ".people[1].name: Tried to read 5 bytes from slice, with 4 byte remaining (at byte 20)"
    );
    assert_eq!(error.path(), Some(".people[1].name"));
    assert_eq!(error.range(), Some(20..20));

    let error = musli::storage::from_slice::<PeopleFrom>(&bytes[..bytes.len() - 3]).unwrap_err();
    assert_eq!(error.path(), None);
}

/// A binary format reports the path and position of a value of the wrong type.
#[test]
fn storage_traced_mismatch() {
    let bytes = musli::storage::to_vec(&people()).unwrap();

    let error = musli::storage::from_slice_traced::<PeopleTo>(&bytes).unwrap_err();
    assert_eq!(
        error.to_string(),
        ".people[0].age: Tried to read 61 bytes from slice, with 10 byte remaining (at byte 17)"
    );
    assert_eq!(error.path(), Some(".people[0].age"));
    assert_eq!(error.range(), Some(17..17));
}

/// A binary format reports the path and position of a value of the wrong type.
#[test]
fn wire_traced_mismatch() {
    let bytes = musli::wire::to_vec(&people()).unwrap();

    let error = musli::wire::from_slice_traced::<PeopleTo>(&bytes).unwrap_err();
    assert_eq!(
        error.to_string(),
        ".people[0].age: Expected length prefix for bytes, but got Continuation (at bytes 16-17)"
    );
    assert_eq!(error.path(), Some(".people[0].age"));
    assert_eq!(error.range(), Some(16..17));

    let error =
        musli::wire::from_slice_traced::<PeopleFrom>(&bytes[..bytes.len() - 1]).unwrap_err();
    assert_eq!(
        error.to_string(),
        ".people[1].age: Tried to read 1 bytes from slice, with 0 byte remaining (at byte 27)"
    );
}

/// Errors collected as other types through a tracing context are decorated as
/// well.
#[test]
fn traced_string_and_capture() {
    const ENCODING: musli::json::Encoding = musli::json::Encoding::new();

    let expected = ".people[1].age: Expected string, found <number> (at bytes 73-74)";

    let cx = context::new().with_trace().with_error::<String>();
    let error = ENCODING
        .from_str_with::<_, PeopleTo>(&cx, JSON_MISMATCH)
        .unwrap_err();
    assert_eq!(error, expected);

    let cx = context::new()
        .with_trace()
        .with_capture::<musli::json::Error>();
    let ErrorMarker { .. } = ENCODING
        .from_str_with::<_, PeopleTo>(&cx, JSON_MISMATCH)
        .unwrap_err();
    let error = cx.unwrap();
    assert_eq!(error.to_string(), expected);
    assert_eq!(error.path(), Some(".people[1].age"));
}

/// A traced error through a fixed buffer is decorated, and degrades into an
/// allocation error rather than failing when the buffer is too small.
#[test]
fn traced_slice_allocator() {
    const ENCODING: musli::json::Encoding = musli::json::Encoding::new();

    let mut buf = ArrayBuffer::<4096>::with_size();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc).with_trace().with_error();
    let error: musli::json::Error<&Slice<'_>> = ENCODING
        .from_str_with::<_, PeopleTo>(&cx, JSON_MISMATCH)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        ".people[1].age: Expected string, found <number> (at bytes 73-74)"
    );

    let mut buf = ArrayBuffer::<8>::with_size();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc).with_trace().with_error();
    let error: musli::json::Error<&Slice<'_>> = ENCODING
        .from_str_with::<_, PeopleTo>(&cx, JSON_MISMATCH)
        .unwrap_err();
    assert_eq!(error.path(), None);
    assert!(!error.to_string().is_empty());
}

#[derive(Debug)]
struct Custom;

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "custom failure")
    }
}

impl core::error::Error for Custom {}

/// `std::io::Error` keeps a traced custom error as its source.
#[test]
fn traced_io_error_source() {
    let cx = context::new().with_trace().with_error::<std::io::Error>();
    (&cx).advance(5);
    let error = (&cx).custom(Custom);

    assert_eq!(error.to_string(), "custom failure (at byte 5)");
    let source = error.source().expect("source");
    assert!(source.downcast_ref::<Custom>().is_some());
}

/// Tracing does not grow the error, which is returned through every level of
/// decoding.
#[test]
fn error_size() {
    let string = size_of::<musli::alloc::String<Global>>();
    assert_eq!(size_of::<musli::json::Error>(), string);
    assert_eq!(size_of::<musli::storage::Error>(), string);
    assert_eq!(size_of::<musli::wire::Error>(), string);
    assert_eq!(size_of::<musli::packed::Error>(), string);
    assert_eq!(size_of::<musli::descriptive::Error>(), string);
}

#[derive(Debug, Decode)]
struct AgeString {
    #[allow(unused)]
    age: String,
}

#[derive(Debug, Decode)]
struct Nested {
    #[allow(unused)]
    inner: AgeString,
}

fn json_range<'de, T>(input: &'de str) -> core::ops::Range<usize>
where
    T: fmt::Debug + Decode<'de, musli::mode::Text, Global>,
{
    musli::json::from_str_traced::<T>(input)
        .unwrap_err()
        .range()
        .unwrap()
}

#[test]
fn json_trace_range_skips_leading_whitespace() {
    // The range starts at the offending value, not at the whitespace before it.
    let input = r#"{"age": 1}"#;
    assert_eq!(&input[8..9], "1");
    assert_eq!(json_range::<AgeString>(input).start, 8);

    let input = r#"{"age":1}"#;
    assert_eq!(&input[7..8], "1");
    assert_eq!(json_range::<AgeString>(input).start, 7);

    let input = "{\"age\" \n\t :   1}";
    let at = input.find('1').unwrap();
    assert_eq!(json_range::<AgeString>(input).start, at);
}

#[test]
fn json_trace_range_nested() {
    let input = r#"{ "inner" : { "age" :   true } }"#;
    let at = input.find("true").unwrap();
    assert_eq!(json_range::<Nested>(input).start, at);
}

#[test]
fn json_trace_range_array() {
    let input = "[\"a\",  \"b\",   2]";
    let at = input.find('2').unwrap();
    let error = musli::json::from_str_traced::<Vec<String>>(input).unwrap_err();
    assert_eq!(error.range().unwrap().start, at);
}

#[test]
fn json_trace_range_char() {
    let input = "   \"ab\"";
    let error = musli::json::from_str_traced::<char>(input).unwrap_err();
    assert_eq!(error.range().unwrap().start, 3);
}
