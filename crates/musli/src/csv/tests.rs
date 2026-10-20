//! Tests for the CSV format.

#![cfg(feature = "std")]

use rust_alloc::string::{String, ToString};
use rust_alloc::vec;
use rust_alloc::vec::Vec;

use crate::alloc::Global;
use crate::context;
use crate::{Decode, Encode};

use super::{Encoding, Terminator};

const DEFAULT: Encoding = Encoding::new();
const NO_HEADERS: Encoding = Encoding::new().with_headers(false);

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Point(i32, i32);

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name_all = "index")]
struct Indexed {
    name: String,
    age: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[musli(crate, name_all = "name")]
enum Color {
    Red,
    Green,
    #[musli(name = "dark-blue")]
    Blue,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
enum Mixed {
    Unit,
    Data(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Everything {
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    u128: u128,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    i128: i128,
    usize: usize,
    isize: isize,
    f32: f32,
    f64: f64,
    bool: bool,
    char: char,
    string: String,
    color: Color,
    some: Option<u32>,
    none: Option<u32>,
    unit: (),
}

fn person(name: &str, age: u32) -> Person {
    Person {
        name: name.to_string(),
        age,
    }
}

/// Encode with the given encoding, and decode it back.
#[track_caller]
fn rt<T>(encoding: Encoding, value: &T, expected: &str) -> T
where
    T: Encode<crate::mode::Text> + for<'de> Decode<'de, crate::mode::Text, Global>,
    T: PartialEq + core::fmt::Debug,
{
    let cx = context::new().with_trace();

    let Ok(string) = encoding.to_string_with(&cx, value) else {
        panic!("Failed to encode {value:?}:\n{}", cx.report());
    };

    assert_eq!(string, expected, "Encoded CSV does not match");

    let Ok(decoded) = encoding.from_str_with::<_, T>(&cx, &string) else {
        panic!("Failed to decode {string:?}:\n{}", cx.report());
    };

    assert_eq!(&decoded, value, "Decoded value does not match");

    let mut cursor = string.as_bytes();

    let Ok(decoded) = encoding.decode_with::<_, _, T>(&cx, &mut cursor) else {
        panic!("Failed to decode cursor {string:?}:\n{}", cx.report());
    };

    assert_eq!(&decoded, value, "Decoded value from cursor does not match");
    assert!(cursor.is_empty(), "Cursor not exhausted: {cursor:?}");
    decoded
}

/// Decode the given input, expecting an error which contains `expected`.
#[track_caller]
fn decode_err<T>(encoding: Encoding, input: &str, expected: &str)
where
    T: for<'de> Decode<'de, crate::mode::Text, Global> + core::fmt::Debug,
{
    match encoding.from_str::<T>(input) {
        Ok(value) => panic!("Expected error decoding {input:?}, but got {value:?}"),
        Err(error) => {
            let error = error.to_string();
            assert!(
                error.contains(expected),
                "Error {error:?} does not contain {expected:?}"
            );
        }
    }
}

/// Encode the given value, expecting an error which contains `expected`.
#[track_caller]
fn encode_err<T>(encoding: Encoding, value: &T, expected: &str)
where
    T: Encode<crate::mode::Text> + core::fmt::Debug,
{
    match encoding.to_string(value) {
        Ok(string) => panic!("Expected error encoding {value:?}, but got {string:?}"),
        Err(error) => {
            let error = error.to_string();
            assert!(
                error.contains(expected),
                "Error {error:?} does not contain {expected:?}"
            );
        }
    }
}

#[test]
fn structs() {
    rt(
        DEFAULT,
        &vec![person("Aristotle", 61), person("Plato", 80)],
        "name,age\nAristotle,61\nPlato,80\n",
    );

    rt(
        NO_HEADERS,
        &vec![
            Indexed {
                name: "Aristotle".to_string(),
                age: 61,
            },
            Indexed {
                name: "Plato".to_string(),
                age: 80,
            },
        ],
        "Aristotle,61\nPlato,80\n",
    );

    rt(
        DEFAULT,
        &vec![Indexed {
            name: "Aristotle".to_string(),
            age: 61,
        }],
        "0,1\nAristotle,61\n",
    );

    // A single struct at the top level is a document with a single record.
    rt(DEFAULT, &person("Aristotle", 61), "name,age\nAristotle,61\n");
    rt(DEFAULT, &Vec::<Person>::new(), "");
}

#[test]
fn tuples() {
    rt(DEFAULT, &vec![(1u32, 2u32), (3, 4)], "0,1\n1,2\n3,4\n");
    rt(NO_HEADERS, &vec![(1u32, 2u32), (3, 4)], "1,2\n3,4\n");
    rt(DEFAULT, &vec![Point(1, -2), Point(3, 4)], "0,1\n1,-2\n3,4\n");
    rt(NO_HEADERS, &vec![Point(1, -2), Point(3, 4)], "1,-2\n3,4\n");
    rt(
        NO_HEADERS,
        &vec![("a".to_string(), Some(1u32), true)],
        "a,1,true\n",
    );
}

#[test]
fn sequences() {
    rt(
        DEFAULT,
        &vec![vec![1u32, 2, 3], vec![4, 5, 6]],
        "0,1,2\n1,2,3\n4,5,6\n",
    );
    rt(
        NO_HEADERS,
        &vec![vec!["a".to_string(), "b".to_string()]],
        "a,b\n",
    );
    rt(
        NO_HEADERS.with_flexible(true),
        &vec![vec![1u32], vec![1, 2, 3]],
        "1\n1,2,3\n",
    );
}

#[test]
fn scalars() {
    rt(DEFAULT, &vec![1u32, 2, 3], "0\n1\n2\n3\n");
    rt(NO_HEADERS, &vec![1u32, 2, 3], "1\n2\n3\n");
    rt(DEFAULT, &42u32, "0\n42\n");
    rt(NO_HEADERS, &42u32, "42\n");
    rt(
        NO_HEADERS,
        &vec!["a".to_string(), String::new(), "c".to_string()],
        "a\n\"\"\nc\n",
    );
    rt(NO_HEADERS, &vec![Some(1u32), None, Some(3)], "1\n\"\"\n3\n");
    rt(NO_HEADERS, &vec![Color::Red, Color::Blue], "Red\ndark-blue\n");
}

#[test]
fn field_values() {
    let value = Everything {
        u8: u8::MAX,
        u16: u16::MAX,
        u32: u32::MAX,
        u64: u64::MAX,
        u128: u128::MAX,
        i8: i8::MIN,
        i16: i16::MIN,
        i32: i32::MIN,
        i64: i64::MIN,
        i128: i128::MIN,
        usize: usize::MAX,
        isize: isize::MIN,
        f32: 1.5,
        f64: -0.25e100,
        bool: true,
        char: 'ä',
        string: "hello, \"world\"".to_string(),
        color: Color::Green,
        some: Some(7),
        none: None,
        unit: (),
    };

    let expected = rust_alloc::format!(
        "u8,u16,u32,u64,u128,i8,i16,i32,i64,i128,usize,isize,f32,f64,bool,char,string,color,some,none,unit\n\
         {},{},{},{},{},{},{},{},{},{},{},{},1.5,-2.5e99,true,ä,\"hello, \"\"world\"\"\",Green,7,,\n",
        u8::MAX,
        u16::MAX,
        u32::MAX,
        u64::MAX,
        u128::MAX,
        i8::MIN,
        i16::MIN,
        i32::MIN,
        i64::MIN,
        i128::MIN,
        usize::MAX,
        isize::MIN,
    );

    rt(DEFAULT, &vec![value], &expected);
}

#[test]
fn floats() {
    rt(
        NO_HEADERS,
        &vec![(0.1f64, f64::INFINITY, f64::NEG_INFINITY)],
        "0.1,inf,-inf\n",
    );

    let decoded: Vec<f64> = NO_HEADERS.from_str("NaN\n").unwrap();
    assert!(decoded[0].is_nan());
}

#[test]
fn mixed_enum() {
    rt(NO_HEADERS, &vec![(Mixed::Unit, 1u32)], "Unit,1\n");
    rt(NO_HEADERS, &vec![Mixed::Unit], "Unit\n");
    encode_err(NO_HEADERS, &vec![(Mixed::Data(1), 1u32)], "unit variants");
}

#[test]
fn terminators() {
    let _ = Terminator::CRLF;
    let _ = decode_err::<Vec<Person>>;
}
