//! Entry points which are handed the whole input reject anything left over,
//! while the reader based `decode` entry points stop at the end of the value so
//! that several values can be read out of one buffer.

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Person {
    name: String,
    age: u32,
}

fn person() -> Person {
    Person {
        name: "Aristotle".to_string(),
        age: 61,
    }
}

macro_rules! binary {
    ($($module:ident),* $(,)?) => {
        $(
            mod $module {
                use super::{Person, person};

                #[test]
                fn exact_input_is_accepted() {
                    let bytes = musli::$module::to_vec(&person()).unwrap();
                    assert_eq!(
                        musli::$module::from_slice::<Person>(&bytes).unwrap(),
                        person()
                    );
                }

                #[test]
                fn trailing_input_is_rejected() {
                    let mut bytes = musli::$module::to_vec(&person()).unwrap();
                    bytes.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
                    assert!(musli::$module::from_slice::<Person>(&bytes).is_err());
                }

                /// Reading through a reader stops at the end of the value, so
                /// several values can be read out of the same buffer.
                //
                // NB: the borrow cannot be removed, since a `&[u8]` is
                // `Copy` and passing it by value would not advance the caller.
                #[allow(clippy::needless_borrows_for_generic_args)]
                #[test]
                fn decode_stops_at_the_end_of_the_value() {
                    let mut bytes = musli::$module::to_vec(&person()).unwrap();
                    let len = bytes.len();
                    bytes.extend_from_slice(&musli::$module::to_vec(&person()).unwrap());
                    bytes.extend_from_slice(&[0xde, 0xad]);

                    let mut reader = &bytes[..];

                    let first: Person = musli::$module::decode(&mut reader).unwrap();
                    assert_eq!(first, person());
                    assert_eq!(reader.len(), bytes.len() - len);

                    let second: Person = musli::$module::decode(&mut reader).unwrap();
                    assert_eq!(second, person());
                    assert_eq!(reader, &[0xde, 0xad]);
                }
            }
        )*
    };
}

binary!(storage, wire, descriptive, packed);

/// JSON documents which are invalid because of what follows the value.
#[test]
fn json_rejects_trailing_input() {
    assert!(musli::json::from_slice::<u32>(b"1 2").is_err());
    assert!(musli::json::from_slice::<u32>(b"1abc").is_err());
    assert!(musli::json::from_slice::<bool>(b"true0").is_err());
    assert!(musli::json::from_slice::<Vec<u32>>(b"[1] xx").is_err());
    assert!(musli::json::from_slice::<Vec<u32>>(b"[1][2]").is_err());

    // JSON forbids leading zeros, which used to read `0` and drop the rest.
    assert!(musli::json::from_slice::<u32>(b"01").is_err());
    assert!(musli::json::from_slice::<u32>(b"00").is_err());
}

/// Whitespace is not part of a JSON value, so it is not trailing input.
#[test]
fn json_allows_surrounding_whitespace() {
    assert_eq!(musli::json::from_str::<u32>("1").unwrap(), 1);
    assert_eq!(musli::json::from_str::<u32>(" 1 ").unwrap(), 1);
    assert_eq!(musli::json::from_str::<u32>("1\n").unwrap(), 1);
    assert_eq!(musli::json::from_str::<u32>("\t1\r\n").unwrap(), 1);
    assert_eq!(
        musli::json::from_str::<Vec<u32>>(" [1, 2] \n").unwrap(),
        [1, 2]
    );
    assert_eq!(
        musli::json::from_str::<Person>(" {\"name\":\"Aristotle\",\"age\":61} ").unwrap(),
        person()
    );
}
