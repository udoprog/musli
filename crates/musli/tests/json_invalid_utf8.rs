#![cfg(feature = "json")]

//! Strings which are not valid UTF-8 must be rejected, including when the
//! string has to be unescaped through the scratch buffer.

use musli::json;

#[test]
fn invalid_utf8_is_rejected() {
    // Borrowed directly out of the input.
    assert!(json::from_slice::<String>(b"\"a\xffb\"").is_err());

    // Unescaped through scratch, with the invalid byte before the escape.
    assert!(json::from_slice::<String>(b"\"a\xffb\\nc\"").is_err());

    // Unescaped through scratch, with the invalid byte after the escape.
    assert!(json::from_slice::<String>(b"\"a\\nb\xffc\"").is_err());

    // A truncated multi-byte sequence at the end of the string.
    assert!(json::from_slice::<String>(b"\"a\xc3\"").is_err());
}

#[test]
fn valid_utf8_is_accepted() {
    let actual = json::from_slice::<String>("\"Aristotle\"".as_bytes()).unwrap();
    assert_eq!(actual, "Aristotle");

    let actual = json::from_slice::<String>("\"är\\nglad\"".as_bytes()).unwrap();
    assert_eq!(actual, "är\nglad");

    let actual = json::from_slice::<String>("\"\\u00e4r\\tglad ✨\"".as_bytes()).unwrap();
    assert_eq!(actual, "är\tglad ✨");
}
