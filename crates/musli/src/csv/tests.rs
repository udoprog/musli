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
    rt(
        DEFAULT,
        &person("Aristotle", 61),
        "name,age\nAristotle,61\n",
    );
    rt(DEFAULT, &Vec::<Person>::new(), "");
}

#[test]
fn tuples() {
    rt(DEFAULT, &vec![(1u32, 2u32), (3, 4)], "0,1\n1,2\n3,4\n");
    rt(NO_HEADERS, &vec![(1u32, 2u32), (3, 4)], "1,2\n3,4\n");
    rt(
        DEFAULT,
        &vec![Point(1, -2), Point(3, 4)],
        "0,1\n1,-2\n3,4\n",
    );
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
    rt(
        NO_HEADERS,
        &vec![Color::Red, Color::Blue],
        "Red\ndark-blue\n",
    );
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

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Optional {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Borrowed<'a> {
    name: &'a str,
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct Nested {
    name: String,
    person: Person,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate)]
struct WithVec {
    values: Vec<u32>,
}

#[test]
fn config_delimiter() {
    let tsv = DEFAULT.with_delimiter(b'\t');

    rt(
        tsv,
        &vec![person("Aristotle, the philosopher", 61)],
        "name\tage\nAristotle, the philosopher\t61\n",
    );

    rt(
        tsv,
        &vec![person("tab\there", 61)],
        "name\tage\n\"tab\there\"\t61\n",
    );

    rt(
        DEFAULT.with_delimiter(b';'),
        &vec![(1.5f64, "a;b".to_string())],
        "0;1\n1.5;\"a;b\"\n",
    );
}

#[test]
fn config_quote() {
    let single = NO_HEADERS.with_quote(b'\'');

    rt(
        single,
        &vec![("it's".to_string(), "a,b".to_string(), "\"".to_string())],
        "'it''s','a,b',\"\n",
    );

    // Double quotes are not special with a different quote character.
    let decoded: Vec<(String, String)> = single.from_str("\"a\",'b''c'\n").unwrap();
    assert_eq!(decoded, [("\"a\"".to_string(), "b'c".to_string())]);
}

#[test]
fn config_terminator() {
    let crlf = DEFAULT.with_terminator(Terminator::CRLF);

    rt(
        crlf,
        &vec![person("Aristotle", 61), person("Plato", 80)],
        "name,age\r\nAristotle,61\r\nPlato,80\r\n",
    );

    let semi = NO_HEADERS.with_terminator(Terminator::Any(b';'));
    rt(semi, &vec![(1u32, 2u32), (3, 4)], "1,2;3,4;");
    // Newlines are not terminators when another terminator is configured, so
    // they are quoted.
    rt(semi, &vec!["a\nb".to_string()], "\"a\nb\";");
    rt(semi, &vec!["a;b".to_string()], "\"a;b\";");

    // Reading accepts LF, CRLF and CR by default.
    let expected = vec![person("Aristotle", 61), person("Plato", 80)];

    for input in [
        "name,age\nAristotle,61\nPlato,80\n",
        "name,age\r\nAristotle,61\r\nPlato,80\r\n",
        "name,age\rAristotle,61\rPlato,80\r",
        "name,age\r\nAristotle,61\nPlato,80",
        "\n\nname,age\n\nAristotle,61\r\n\r\nPlato,80\n\n",
    ] {
        let decoded: Vec<Person> = DEFAULT.from_str(input).unwrap();
        assert_eq!(decoded, expected, "{input:?}");
        let decoded: Vec<Person> = crlf.from_str(input).unwrap();
        assert_eq!(decoded, expected, "{input:?}");
    }
}

#[test]
fn config_flexible() {
    let flexible = DEFAULT.with_flexible(true);

    let input = "name,age\nAristotle\nPlato,80,extra\n";

    let decoded: Vec<Optional> = flexible.from_str(input).unwrap();

    assert_eq!(
        decoded,
        [
            Optional {
                name: "Aristotle".to_string(),
                age: None,
            },
            Optional {
                name: "Plato".to_string(),
                age: Some(80),
            },
        ]
    );

    decode_err::<Vec<Optional>>(DEFAULT, input, "Record 1 has 1 fields, but expected 2");
    decode_err::<Vec<Optional>>(
        DEFAULT,
        "name,age\nPlato,80,extra\n",
        "Record 1 has more fields than the 2 in the header row",
    );

    // Writing records of differing lengths.
    encode_err(
        NO_HEADERS,
        &vec![vec![1u32], vec![1, 2]],
        "Record 2 has more than the expected 1 fields",
    );
    encode_err(
        DEFAULT,
        &vec![vec![1u32, 2], vec![1]],
        "Record 2 has 1 fields, but expected 2",
    );
}

#[test]
fn headers_reordered() {
    let decoded: Vec<Person> = DEFAULT
        .from_str("age,name\n61,Aristotle\n80,Plato\n")
        .unwrap();
    assert_eq!(decoded, [person("Aristotle", 61), person("Plato", 80)]);
}

#[test]
fn headers_extra() {
    let decoded: Vec<Person> = DEFAULT
        .from_str("id,age,city,name,\n1,61,Stagira,Aristotle,\n2,80,Athens,Plato,x\n")
        .unwrap();
    assert_eq!(decoded, [person("Aristotle", 61), person("Plato", 80)]);
}

#[test]
fn headers_missing() {
    let decoded: Vec<Optional> = DEFAULT.from_str("name\nAristotle\nPlato\n").unwrap();

    assert_eq!(
        decoded,
        [
            Optional {
                name: "Aristotle".to_string(),
                age: None,
            },
            Optional {
                name: "Plato".to_string(),
                age: None,
            },
        ]
    );

    decode_err::<Vec<Person>>(DEFAULT, "name\nAristotle\n", "age");
}

#[test]
fn headers_quoted() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, name_all = "name")]
    struct Weird {
        #[musli(name = "a,b")]
        comma: u32,
        #[musli(name = "say \"hi\"")]
        quote: u32,
    }

    rt(
        DEFAULT,
        &vec![Weird { comma: 1, quote: 2 }],
        "\"a,b\",\"say \"\"hi\"\"\"\n1,2\n",
    );
}

#[test]
fn headers_skipped_for_tuples() {
    // A header row is always skipped when reading with headers enabled.
    let decoded: Vec<(String, u32)> = DEFAULT.from_str("name,age\nAristotle,61\n").unwrap();
    assert_eq!(decoded, [("Aristotle".to_string(), 61)]);

    // Without headers, named fields can't be matched.
    decode_err::<Vec<Person>>(NO_HEADERS, "Aristotle,61\n", "name");
}

#[test]
fn quoting() {
    let values = vec![
        "plain".to_string(),
        "with,comma".to_string(),
        "with \"quote\"".to_string(),
        "\"".to_string(),
        "\"\"".to_string(),
        "line\nbreak".to_string(),
        "carriage\rreturn".to_string(),
        "crlf\r\nbreak".to_string(),
        " spaces ".to_string(),
        String::new(),
        "unicode ✓".to_string(),
    ];

    rt(
        NO_HEADERS,
        &vec![values],
        "plain,\"with,comma\",\"with \"\"quote\"\"\",\"\"\"\",\"\"\"\"\"\",\"line\nbreak\",\"carriage\rreturn\",\"crlf\r\nbreak\", spaces ,,unicode ✓\n",
    );

    // Empty fields.
    rt(NO_HEADERS, &vec![(String::new(), String::new())], ",\n");
    rt(NO_HEADERS, &vec![(String::new(),)], "\"\"\n");

    // Quotes inside of unquoted fields are taken literally, and quoted fields
    // may contain anything.
    let decoded: Vec<(String, String, String)> =
        NO_HEADERS.from_str("a\"b,\"c\nd\",\"\"\n").unwrap();

    assert_eq!(
        decoded,
        [("a\"b".to_string(), "c\nd".to_string(), String::new())]
    );
}

#[test]
fn borrowed() {
    let input = "name,age\nAristotle,61\n\"Plato\",80\n";
    let decoded: Vec<Borrowed<'_>> = DEFAULT.from_str(input).unwrap();

    assert_eq!(
        decoded,
        [
            Borrowed {
                name: "Aristotle",
                age: 61,
            },
            Borrowed {
                name: "Plato",
                age: 80,
            },
        ]
    );

    let name = decoded[0].name.as_ptr() as usize;
    let start = input.as_ptr() as usize;
    assert!(name >= start && name < start + input.len());

    // A field with escaped quotes cannot be borrowed.
    assert!(
        DEFAULT
            .from_str::<Vec<Borrowed<'_>>>("name,age\n\"a\"\"b\",1\n")
            .is_err()
    );
}

#[test]
fn malformed_unterminated() {
    decode_err::<Vec<(String, String)>>(NO_HEADERS, "a,\"b\n", "Unterminated quoted field");
    decode_err::<Vec<Person>>(DEFAULT, "name,\"age\n", "Unterminated quoted field");
    decode_err::<Vec<(String, String)>>(
        NO_HEADERS,
        "a,\"b\"c\n",
        "Expected delimiter or end of record after closing quote",
    );
}

#[test]
fn malformed_field_count() {
    decode_err::<Vec<(u32, u32)>>(
        NO_HEADERS,
        "1,2\n3\n",
        "Record 2 has 1 fields, but more were expected",
    );
    decode_err::<Vec<(u32, u32)>>(
        NO_HEADERS,
        "1,2\n3,4,5\n",
        "Record 2 has 3 fields, but expected 2",
    );
    decode_err::<Vec<Vec<u32>>>(
        NO_HEADERS,
        "1,2\n3,4,5\n",
        "Record 2 has 3 fields, but expected 2",
    );
    decode_err::<Vec<Person>>(
        DEFAULT,
        "name,age\nAristotle\n",
        "Record 1 has 1 fields, but expected 2",
    );
    decode_err::<Vec<u32>>(NO_HEADERS, "1,2\n", "Record 1 has 2 fields, but expected 1");
    decode_err::<Person>(
        DEFAULT,
        "",
        "Expected a record, but reached the end of the input",
    );
    decode_err::<Person>(
        DEFAULT,
        "name,age\n",
        "Expected a record, but reached the end of the input",
    );
}

#[test]
fn malformed_values() {
    decode_err::<Vec<Person>>(
        DEFAULT,
        "name,age\nAristotle,sixty\n",
        "Expected an unsigned 32-bit integer, but found \"sixty\"",
    );
    decode_err::<Vec<Person>>(
        DEFAULT,
        "name,age\nAristotle,-1\n",
        "Expected an unsigned 32-bit integer, but found \"-1\"",
    );
    decode_err::<Vec<(u8,)>>(
        NO_HEADERS,
        "256\n",
        "Expected an unsigned 8-bit integer, but found \"256\"",
    );
    decode_err::<Vec<(f64,)>>(NO_HEADERS, "1.5.2\n", "Expected a 64-bit float");
    decode_err::<Vec<(bool,)>>(NO_HEADERS, "yes\n", "Expected a boolean");
    decode_err::<Vec<(char,)>>(NO_HEADERS, "ab\n", "Expected a single character");
    decode_err::<Vec<(Color,)>>(NO_HEADERS, "Purple\n", "Purple");
    decode_err::<Vec<((),)>>(NO_HEADERS, "x\n", "Expected an empty field");

    // Invalid UTF-8.
    assert!(NO_HEADERS.from_slice::<Vec<String>>(b"\xff\n").is_err());
    assert!(
        DEFAULT
            .from_slice::<Vec<Person>>(b"name,\xff\nA,1\n")
            .is_err()
    );
}

#[test]
fn nested_containers() {
    encode_err(
        DEFAULT,
        &vec![Nested {
            name: "a".to_string(),
            person: person("b", 1),
        }],
        "CSV fields cannot contain containers",
    );

    encode_err(
        DEFAULT,
        &vec![WithVec { values: vec![1, 2] }],
        "CSV fields cannot contain containers",
    );

    encode_err(
        NO_HEADERS,
        &vec![vec![vec![1u32]]],
        "CSV fields cannot contain containers",
    );

    decode_err::<Vec<Nested>>(
        DEFAULT,
        "name,person\na,b\n",
        "CSV fields cannot contain containers",
    );

    decode_err::<Vec<WithVec>>(
        DEFAULT,
        "values\n1\n",
        "CSV fields cannot contain containers",
    );

    encode_err(
        NO_HEADERS,
        &vec![(Vec::<(u32,)>::new(),)],
        "CSV fields cannot contain",
    );
    encode_err(NO_HEADERS, &vec![Vec::<u32>::new()], "at least one field");
}

#[test]
fn trailing_input() {
    // A single record at the top level must be the only record.
    decode_err::<Person>(DEFAULT, "name,age\nA,1\nB,2\n", "Trailing input");
    // But a tuple of records skips the remaining ones.
    let (a, b): (Person, Person) = DEFAULT.from_str("name,age\nA,1\nB,2\nC,3\n").unwrap();
    assert_eq!((a, b), (person("A", 1), person("B", 2)));
}

#[test]
fn options() {
    let decoded: Option<Person> = DEFAULT.from_str("").unwrap();
    assert_eq!(decoded, None);

    rt(
        DEFAULT,
        &Some(person("Aristotle", 61)),
        "name,age\nAristotle,61\n",
    );
    rt(DEFAULT, &None::<Person>, "");

    // Records which are options.
    rt(
        NO_HEADERS,
        &vec![Some((1u32, 2u32)), None, Some((3, 4))],
        "1,2\n\"\"\n3,4\n",
    );
}

#[test]
fn binary_mode() {
    const BINARY: Encoding<crate::mode::Binary> = Encoding::new().with_mode();

    // Fields are identified by index in binary mode.
    let string = BINARY.to_string(&vec![person("Aristotle", 61)]).unwrap();
    assert_eq!(string, "0,1\nAristotle,61\n");
    let decoded: Vec<Person> = BINARY.from_str(&string).unwrap();
    assert_eq!(decoded, [person("Aristotle", 61)]);

    let decoded: Vec<Person> = BINARY.from_str("1,0\n61,Aristotle\n").unwrap();
    assert_eq!(decoded, [person("Aristotle", 61)]);
}

#[test]
fn writer() {
    let mut data = Vec::new();
    let mut writer = DEFAULT.writer(&mut data);
    writer.serialize(&person("Aristotle", 61)).unwrap();
    writer
        .serialize(&person("Plato, the philosopher", 80))
        .unwrap();
    assert_eq!(
        data,
        b"name,age\nAristotle,61\n\"Plato, the philosopher\",80\n"
    );

    let mut writer = DEFAULT.writer(&mut data);
    writer.serialize(&(1u32, 2u32)).unwrap();
    // Extra fields are detected before they are written, but missing fields
    // are only detected after the record has been written.
    assert!(writer.serialize(&(1u32, 2u32, 3u32)).is_err());
    assert!(writer.serialize(&(1u32,)).is_err());

    let mut data = Vec::new();
    let mut writer = NO_HEADERS.writer(&mut data);
    writer.serialize(&(1u32, "a")).unwrap();
    writer.serialize(&[2u32, 3]).unwrap();
    assert_eq!(writer.get_ref().as_slice(), b"1,a\n2,3\n");
    writer.serialize(&Color::Blue).unwrap_err();

    let mut data = Vec::new();
    let mut writer = DEFAULT.writer(&mut data);
    writer.serialize(&Color::Blue).unwrap();
    writer.serialize(&Some(Color::Red)).unwrap();
    writer.serialize(&None::<Color>).unwrap();
    assert_eq!(data, b"0\ndark-blue\nRed\n\"\"\n");

    // Writing into a slice.
    let mut buf = [0u8; 32];
    let mut writer = DEFAULT.writer(&mut buf[..]);
    writer.serialize(&(1u32, 2u32)).unwrap();
    let remaining = writer.finish().unwrap();
    assert_eq!(&buf[..32 - remaining], b"0,1\n1,2\n");

    // Writing to a slice which is too small.
    let mut buf = [0u8; 4];
    let mut writer = DEFAULT.writer(&mut buf[..]);
    assert!(writer.serialize(&(1u32, 2u32)).is_err());
}

#[test]
fn reader() {
    let input = "name,age\nAristotle,61\nPlato,80\n";

    let mut reader = DEFAULT.reader(input);
    let headers = reader.headers().unwrap().unwrap();
    assert!(headers.iter().eq(["name", "age"]));
    assert_eq!(rust_alloc::format!("{headers:?}"), "[\"name\", \"age\"]");

    let people = reader
        .deserialize::<Borrowed<'_>>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(
        people,
        [
            Borrowed {
                name: "Aristotle",
                age: 61,
            },
            Borrowed {
                name: "Plato",
                age: 80,
            },
        ]
    );

    assert!(reader.is_done());
    assert_eq!(reader.next_record::<Person>().unwrap(), None);

    // Headers are read implicitly.
    let mut reader = DEFAULT.reader(input);
    assert_eq!(
        reader.next_record::<(String, u32)>().unwrap(),
        Some(("Aristotle".to_string(), 61))
    );
    assert!(
        reader
            .headers()
            .unwrap()
            .unwrap()
            .iter()
            .eq(["name", "age"])
    );

    // Without headers.
    let mut reader = NO_HEADERS.reader(input);
    assert!(reader.headers().unwrap().is_none());
    assert_eq!(
        reader.next_record::<(String, String)>().unwrap(),
        Some(("name".to_string(), "age".to_string()))
    );

    // Errors stop iteration.
    let mut reader = DEFAULT.reader("name,age\nA,1\nB,x\nC,3\n");
    let results = reader.deserialize::<Person>().collect::<Vec<_>>();
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert!(results[1].is_err());

    // Field counts are checked across records.
    let mut reader = DEFAULT.reader("a,b\n1,2\n3\n");
    assert!(reader.next_record::<Vec<u32>>().unwrap().is_some());
    assert!(reader.next_record::<Vec<u32>>().is_err());

    // Empty input.
    let mut reader = DEFAULT.reader("");
    assert!(reader.headers().unwrap().is_none());
    assert!(reader.next_record::<Person>().unwrap().is_none());
}

#[test]
fn reader_unused_fields() {
    // Decoders which don't use the field they are given still advance.
    #[derive(Debug, PartialEq)]
    struct Second(u32);

    impl<'de, M, A> Decode<'de, M, A> for Second
    where
        A: crate::Allocator,
    {
        #[inline]
        fn decode<D>(decoder: D) -> Result<Self, D::Error>
        where
            D: crate::Decoder<'de, Mode = M, Allocator = A>,
        {
            use crate::de::SequenceDecoder;

            decoder.decode_sequence(|seq| {
                // Ignore the first field.
                let _ = seq.decode_next()?;
                seq.next().map(Second)
            })
        }
    }

    let decoded: Vec<Second> = NO_HEADERS.from_str("1,2\n3,4\n").unwrap();
    assert_eq!(decoded, [Second(2), Second(4)]);
}
