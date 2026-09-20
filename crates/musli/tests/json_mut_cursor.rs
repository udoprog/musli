#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(Text, tag = "type")]
enum BorrowedTarget<'a> {
    #[musli(Text, name = "port")]
    Port { name: &'a str },
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(Text, tag = "type")]
enum OwnedTarget {
    #[musli(Text, name = "port")]
    Port { name: String },
}

// NB: the mutable references are load-bearing: both `&str` and `&[u8]` are
// `Copy`, so passing either cursor by value would not test its advancement.
#[allow(clippy::needless_borrows_for_generic_args)]
fn assert_tag_error_matches_immutable(input: &str) {
    let immutable = musli::json::from_str::<BorrowedTarget<'_>>(input)
        .unwrap_err()
        .to_string();

    let mut string = input;
    let mutable: Result<BorrowedTarget<'_>, _> = musli::json::decode(&mut string);
    assert_eq!(mutable.unwrap_err().to_string(), immutable);
    assert!(core::str::from_utf8(string.as_bytes()).is_ok());

    let immutable = musli::json::from_slice::<BorrowedTarget<'_>>(input.as_bytes())
        .unwrap_err()
        .to_string();
    let mut bytes = input.as_bytes();
    let mutable: Result<BorrowedTarget<'_>, _> = musli::json::decode(&mut bytes);
    assert_eq!(mutable.unwrap_err().to_string(), immutable);
}

#[test]
fn mutable_cursors_match_immutable_missing_tag_error() {
    assert_tag_error_matches_immutable(r#"{"name":"value"}"#);
}

#[test]
fn mutable_cursors_match_immutable_unknown_tag_error() {
    assert_tag_error_matches_immutable(r#"{"name":"value","type":"unknown"}"#);
}

#[test]
fn mutable_cursors_match_immutable_malformed_tag_error() {
    assert_tag_error_matches_immutable(r#"{"name":"value","type":}"#);
}

#[test]
fn mutable_cursors_match_immutable_truncated_tag_error_and_keep_utf8() {
    assert_tag_error_matches_immutable(r#"{"name":"välue","type":"port"#);
}

#[test]
fn mutable_byte_cursor_borrows_multibyte_string_after_tag_lookahead() {
    let input =
        r#"{"nested":{"labels":["ignored",{"deep":true}]},"name":"välue","type":"port"} suffix"#;
    let input_start = input.as_ptr() as usize;
    let input_end = input_start + input.len();
    let mut bytes = input.as_bytes();

    // NB: this mutable reference selects the cursor parser under test.
    #[allow(clippy::needless_borrows_for_generic_args)]
    let decoded: BorrowedTarget<'_> = musli::json::decode(&mut bytes).unwrap();

    assert_eq!(decoded, BorrowedTarget::Port { name: "välue" });
    let BorrowedTarget::Port { name } = decoded;
    assert!((input_start..input_end).contains(&(name.as_ptr() as usize)));
    assert_eq!(bytes, b" suffix");
}

#[test]
fn mutable_string_cursor_borrows_multibyte_string_after_tag_lookahead() {
    let input =
        r#"{"nested":{"labels":["ignored",{"deep":true}]},"name":"välue","type":"port"} suffix"#;
    let input_start = input.as_ptr() as usize;
    let input_end = input_start + input.len();
    let mut string = input;

    // NB: this mutable reference selects the cursor parser under test.
    #[allow(clippy::needless_borrows_for_generic_args)]
    let decoded: BorrowedTarget<'_> = musli::json::decode(&mut string).unwrap();

    assert_eq!(decoded, BorrowedTarget::Port { name: "välue" });
    let BorrowedTarget::Port { name } = decoded;
    assert!((input_start..input_end).contains(&(name.as_ptr() as usize)));
    assert_eq!(string, " suffix");
}

#[test]
fn mutable_byte_cursor_decodes_escaped_string_after_tag_lookahead() {
    let input =
        r#"{"nested":{"labels":["ignored",{"deep":true}]},"name":"väl\nue","type":"port"} suffix"#;
    let mut bytes = input.as_bytes();

    // NB: this mutable reference selects the cursor parser under test.
    #[allow(clippy::needless_borrows_for_generic_args)]
    let decoded: OwnedTarget = musli::json::decode(&mut bytes).unwrap();

    assert_eq!(
        decoded,
        OwnedTarget::Port {
            name: String::from("väl\nue"),
        }
    );
    assert_eq!(bytes, b" suffix");
}

#[test]
fn mutable_string_cursor_decodes_escaped_string_after_tag_lookahead() {
    let input =
        r#"{"nested":{"labels":["ignored",{"deep":true}]},"name":"väl\nue","type":"port"} suffix"#;
    let mut string = input;

    // NB: this mutable reference selects the cursor parser under test.
    #[allow(clippy::needless_borrows_for_generic_args)]
    let decoded: OwnedTarget = musli::json::decode(&mut string).unwrap();

    assert_eq!(
        decoded,
        OwnedTarget::Port {
            name: String::from("väl\nue"),
        }
    );
    assert_eq!(string, " suffix");
}

// NB: the mutable reference is load-bearing; see assert_tag_error_matches_immutable.
#[allow(clippy::needless_borrows_for_generic_args)]
fn assert_invalid_utf8_error_matches_immutable(input: &[u8]) {
    let immutable = musli::json::from_slice::<OwnedTarget>(input)
        .unwrap_err()
        .to_string();
    let mut cursor = input;
    let mutable: Result<OwnedTarget, _> = musli::json::decode(&mut cursor);
    assert_eq!(mutable.unwrap_err().to_string(), immutable);
}

#[test]
fn mutable_byte_cursor_matches_immutable_invalid_utf8_error() {
    assert_invalid_utf8_error_matches_immutable(b"{\"name\":\"a\xffb\",\"type\":\"port\"}");
}

#[test]
fn mutable_byte_cursor_matches_immutable_invalid_utf8_before_escape_error() {
    assert_invalid_utf8_error_matches_immutable(b"{\"name\":\"a\xffb\\nc\",\"type\":\"port\"}");
}

#[test]
fn mutable_byte_cursor_matches_immutable_invalid_utf8_after_escape_error() {
    assert_invalid_utf8_error_matches_immutable(b"{\"name\":\"a\\nb\xffc\",\"type\":\"port\"}");
}

#[test]
fn mutable_byte_cursor_matches_immutable_truncated_multibyte_utf8_error() {
    assert_invalid_utf8_error_matches_immutable(b"{\"name\":\"a\xc3\",\"type\":\"port\"}");
}
