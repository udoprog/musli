#![cfg(feature = "test")]

//! Decoding JSON from a `&mut &str` cursor must leave the cursor holding valid
//! UTF-8 even when decoding fails part of the way into a multibyte character.
//!
//! Each test covers one independent error path, so that a failure in one
//! cannot hide missing coverage in another.

use musli::{Decode, Encode};

/// A struct with no fields, so that every field in the input is skipped.
#[derive(Debug, Encode, Decode)]
struct Empty {}

/// Decode `input` through a `&mut &str` cursor, expect it to fail, and assert
/// that the cursor still holds valid UTF-8.
///
/// The `&mut` borrow selects the `&mut &str` cursor parser; passing the `&str`
/// by value would decode through a copy and never observe the cursor.
#[allow(clippy::needless_borrows_for_generic_args)]
fn assert_cursor_utf8<T>(input: &str)
where
    T: core::fmt::Debug + for<'de> Decode<'de, musli::mode::Text, musli::alloc::Global>,
{
    let mut s = input;
    let result: Result<T, _> = musli::json::decode(&mut s);
    assert!(
        result.is_err(),
        "{input:?}: expected an error, got {result:?}"
    );
    assert!(
        core::str::from_utf8(s.as_bytes()).is_ok(),
        "{input:?}: cursor left with invalid UTF-8: {:?}",
        s.as_bytes()
    );
    assert!(
        input.as_bytes().ends_with(s.as_bytes()),
        "{input:?}: cursor is not a suffix of the input"
    );
}

#[test]
fn parse_exact_true_keeps_utf8() {
    // `true` reads four bytes: 74 C3 A9 E2, ending inside `€`.
    assert_cursor_utf8::<bool>("t\u{e9}\u{20ac}");
}

#[test]
fn parse_exact_false_keeps_utf8() {
    assert_cursor_utf8::<bool>("f\u{e9}\u{20ac}");
}

#[test]
fn parse_exact_null_keeps_utf8() {
    assert_cursor_utf8::<Option<u32>>("n\u{e9}\u{20ac}");
}

#[test]
fn string_escape_error_keeps_utf8() {
    // The byte after the backslash is the lead byte of `é`.
    assert_cursor_utf8::<String>("\"\\\u{e9}\"");
}

#[test]
fn skip_string_escape_error_keeps_utf8() {
    assert_cursor_utf8::<Empty>("{\"x\":\"\\\u{e9}\"}");
}

#[test]
fn skip_parse_exact_keeps_utf8() {
    assert_cursor_utf8::<Empty>("{\"x\":t\u{e9}\u{20ac}}");
}

#[test]
fn number_error_keeps_utf8() {
    // The number error is reported one byte past `-`, inside `é`.
    assert_cursor_utf8::<u32>("-\u{e9}");
    assert_cursor_utf8::<i64>("-\u{e9}");
    assert_cursor_utf8::<f64>("-\u{e9}");
}

#[test]
fn skip_number_error_keeps_utf8() {
    assert_cursor_utf8::<Empty>("{\"x\":-\u{e9}}");
}
