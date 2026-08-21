use rust_alloc::collections::BTreeMap;
use rust_alloc::string::{String, ToString};
use rust_alloc::vec;
use rust_alloc::vec::Vec;

use crate::alloc::Global;
use crate::mode::Binary;
use crate::value::Value;
use crate::{Decode, Encode};

use super::tag::{ARRAY, OBJECT, TEXT, TEXTRAW, write_header};
use super::{Encoding, from_slice, to_vec};

const ENCODING: Encoding = Encoding::new();

/// Wrap `payload` in the header of a container of the given type.
///
/// Used to hand-build documents which no Rust type would produce.
#[track_caller]
fn container(kind: u8, payload: &[u8]) -> Vec<u8> {
    let cx = crate::context::new();
    let mut out = Vec::new();
    write_header(&cx, &mut out, kind, payload.len()).unwrap();
    out.extend_from_slice(payload);
    out
}

#[test]
fn primitives() {
    assert_eq!(to_vec(&()).unwrap(), [0x00]);
    assert_eq!(to_vec(&true).unwrap(), [0x01]);
    assert_eq!(to_vec(&false).unwrap(), [0x02]);
    assert_eq!(to_vec(&Option::<u32>::None).unwrap(), [0x00]);
    assert_eq!(to_vec(&Some(1u32)).unwrap(), [0x13, b'1']);
}

#[test]
fn integers() {
    // The element type of an integer is INT (3), and the size of its ASCII
    // payload goes in the upper four bits of the header byte.
    assert_eq!(to_vec(&0u32).unwrap(), [0x13, b'0']);
    assert_eq!(to_vec(&-1i32).unwrap(), [0x23, b'-', b'1']);
    assert_eq!(to_vec(&123456789u32).unwrap(), *b"\x93123456789".as_slice());

    // A payload of 20 digits no longer fits in four bits, so the size moves
    // into a byte of its own.
    assert_eq!(
        to_vec(&u64::MAX).unwrap(),
        *b"\xc3\x1418446744073709551615".as_slice()
    );
    assert_eq!(
        to_vec(&i64::MIN).unwrap(),
        *b"\xc3\x14-9223372036854775808".as_slice()
    );

    assert_eq!(
        from_slice::<u64>(&to_vec(&u64::MAX).unwrap()).unwrap(),
        u64::MAX
    );
    assert_eq!(
        from_slice::<i64>(&to_vec(&i64::MIN).unwrap()).unwrap(),
        i64::MIN
    );
}

#[test]
fn floats() {
    assert_eq!(to_vec(&1.0f64).unwrap(), [0x35, b'1', b'.', b'0']);

    // The infinities are spelled the way SQLite spells them, as an overflowing
    // exponent, which is canonical JSON.
    assert_eq!(to_vec(&f64::INFINITY).unwrap(), *b"\x559e999".as_slice());
    assert_eq!(
        to_vec(&f64::NEG_INFINITY).unwrap(),
        *b"\x65-9e999".as_slice()
    );
    // NaN has no JSON representation, so the JSON5 spelling is used.
    assert_eq!(to_vec(&f64::NAN).unwrap(), *b"\x36NaN".as_slice());

    assert!(
        from_slice::<f64>(&to_vec(&f64::NAN).unwrap())
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        from_slice::<f64>(&to_vec(&f64::INFINITY).unwrap()).unwrap(),
        f64::INFINITY
    );
    assert_eq!(
        from_slice::<f64>(&to_vec(&f64::NEG_INFINITY).unwrap()).unwrap(),
        f64::NEG_INFINITY
    );
    assert_eq!(from_slice::<f32>(b"\x559e999").unwrap(), f32::INFINITY);
    // The JSON5 spellings which SQLite accepts on input are understood too.
    assert_eq!(from_slice::<f64>(b"\x86Infinity").unwrap(), f64::INFINITY);
    assert_eq!(
        from_slice::<f32>(&to_vec(&1.5e10f32).unwrap()).unwrap(),
        1.5e10
    );
}

#[test]
fn strings() {
    // A string which needs no escaping to be rendered as JSON is TEXT (7).
    assert_eq!(to_vec("abc").unwrap(), *b"\x37abc".as_slice());
    // One which does is TEXTRAW (10), whose payload is the string verbatim.
    assert_eq!(to_vec("a\"b").unwrap(), *b"\x3aa\"b".as_slice());
    assert_eq!(to_vec("a\nb").unwrap(), *b"\x3aa\nb".as_slice());

    assert_eq!(from_slice::<String>(b"\x3aa\"b").unwrap(), "a\"b");
    assert_eq!(to_vec(&'a').unwrap(), *b"\x17a".as_slice());
}

#[test]
fn containers() {
    // ARRAY is 11 and OBJECT is 12.
    assert_eq!(
        to_vec(&vec![1u32, 2]).unwrap(),
        [0x4b, 0x13, b'1', 0x13, b'2']
    );
    assert_eq!(to_vec(&Vec::<u32>::new()).unwrap(), [0x0b]);

    #[derive(Encode)]
    #[musli(crate, name_all = "name")]
    struct Person {
        name: &'static str,
    }

    assert_eq!(
        ENCODING.to_vec(&Person { name: "Bob" }).unwrap(),
        *b"\x9c\x47name\x37Bob".as_slice()
    );
}

/// Variants are externally tagged, so they are encoded as an object with a
/// single entry.
#[test]
fn variants() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, name_all = "name")]
    enum Enum {
        Empty,
        Tuple(u32, u32),
        Struct { field: u32 },
    }

    assert_eq!(
        ENCODING.to_vec(&Enum::Empty).unwrap(),
        // A unit variant carries an empty object as its data.
        *b"\x7c\x57Empty\x0c".as_slice()
    );
    // The fields of a tuple variant are named by their index, which as an
    // object key is the string it is spelled as.
    assert_eq!(
        ENCODING.to_vec(&Enum::Tuple(1, 2)).unwrap(),
        *b"\xcc\x0f\x57Tuple\x8c\x170\x131\x171\x132".as_slice()
    );

    for value in [Enum::Empty, Enum::Tuple(1, 2), Enum::Struct { field: 3 }] {
        let bytes = ENCODING.to_vec(&value).unwrap();
        assert_eq!(ENCODING.from_slice::<Enum>(&bytes).unwrap(), value);
    }
}

/// Decode the escaped string types, which musli never writes but which SQLite
/// produces when it converts JSON text to JSONB.
#[test]
fn decode_escaped_strings() {
    // TEXTJ (8), as produced by `SELECT jsonb('"a\nb"')`.
    assert_eq!(from_slice::<String>(b"\x48a\\nb").unwrap(), "a\nb");

    // A surrogate pair spelled out with `\u` escapes.
    assert_eq!(
        from_slice::<String>(b"\xc8\x0c\\ud83d\\ude00").unwrap(),
        "\u{1f600}"
    );

    // TEXT5 (9) additionally carries the JSON5 escapes.
    assert_eq!(from_slice::<String>(b"\x89a\\x41\\'b").unwrap(), "aA'b");
    // A line continuation contributes nothing to the string.
    assert_eq!(from_slice::<String>(b"\x49a\\\nb").unwrap(), "ab");
    // An escaped character which needs no escaping stands for itself.
    assert_eq!(from_slice::<String>(b"\x49a\\qb").unwrap(), "aqb");
    // Escaping a multi-byte character only escapes its leading byte.
    assert_eq!(
        from_slice::<String>(&[0x59, b'a', b'\\', 0xc3, 0xa9, b'b']).unwrap(),
        "aéb"
    );
    // Which is not the case for the strictly RFC 8259 escapes of TEXTJ.
    assert!(from_slice::<String>(b"\x48a\\qb").is_err());
}

/// Decode the JSON5 number types, which musli only writes for the values which
/// canonical JSON cannot represent.
#[test]
fn decode_json5_numbers() {
    // INT5 (4), as produced by `SELECT jsonb('0x1f')` in JSON5 mode.
    assert_eq!(from_slice::<u32>(b"\x440x1f").unwrap(), 31);
    assert_eq!(from_slice::<i32>(b"\x54-0x1f").unwrap(), -31);
    // FLOAT5 (6) with a leading dot.
    assert_eq!(from_slice::<f64>(b"\x26.5").unwrap(), 0.5);
    // A float payload decoded as a float, and an integer payload as well.
    assert_eq!(from_slice::<f64>(b"\x13\x37").unwrap(), 7.0);
}

#[test]
fn large_payloads() {
    let string = "x".repeat(300);
    let bytes = to_vec(string.as_str()).unwrap();
    // A payload size of 300 needs the two byte big-endian form.
    assert_eq!(&bytes[..3], &[0xd7, 0x01, 0x2c]);
    assert_eq!(from_slice::<String>(&bytes).unwrap(), string);

    let string = "y".repeat(70000);
    let bytes = to_vec(string.as_str()).unwrap();
    // And 70000 needs the four byte one.
    assert_eq!(&bytes[..5], &[0xe7, 0x00, 0x01, 0x11, 0x70]);
    assert_eq!(from_slice::<String>(&bytes).unwrap(), string);
}

#[test]
fn roundtrip_nested() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate)]
    struct Inner {
        values: Vec<Option<String>>,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate)]
    struct Outer {
        inner: Vec<Inner>,
        entries: Vec<(String, u32)>,
    }

    let expected = Outer {
        inner: vec![
            Inner {
                values: vec![Some("a\"b".to_string()), None],
            },
            Inner { values: vec![] },
        ],
        entries: vec![("a".to_string(), 1)],
    };

    let bytes = ENCODING.to_vec(&expected).unwrap();
    let actual: Outer = ENCODING.from_slice(&bytes).unwrap();
    assert_eq!(expected, actual);
}

/// Unknown fields can be skipped over without looking at their contents, since
/// every element knows the size of its payload.
#[test]
fn skip_unknown() {
    #[derive(Encode)]
    #[musli(crate, name_all = "name")]
    struct Full {
        a: u32,
        b: Vec<Vec<u32>>,
        c: String,
    }

    #[derive(Debug, PartialEq, Decode)]
    #[musli(crate, name_all = "name")]
    struct Partial {
        c: String,
    }

    let bytes = ENCODING
        .to_vec(&Full {
            a: 1,
            b: vec![vec![1, 2], vec![3]],
            c: "hello".to_string(),
        })
        .unwrap();

    assert_eq!(
        ENCODING.from_slice::<Partial>(&bytes).unwrap(),
        Partial {
            c: "hello".to_string()
        }
    );
}

/// Every size class of the header, from the four bits inline in the header byte
/// up to the four byte big-endian form.
#[test]
fn header_size_classes() {
    // Up to and including eleven bytes the payload size is the upper four bits
    // of the header byte, so the header is a single byte.
    for len in 0..=11usize {
        let string = "x".repeat(len);
        let bytes = to_vec(string.as_str()).unwrap();
        assert_eq!(bytes.len(), len + 1, "payload of {len} bytes");
        assert_eq!(bytes[0], (len as u8) << 4 | TEXT, "payload of {len} bytes");
        assert_eq!(from_slice::<String>(&bytes).unwrap(), string);
    }

    // Beyond that it moves into one, two or four bytes following the header
    // byte, which say so.
    for (len, header, header_len) in [
        (12usize, 0xc0 | TEXT, 2usize),
        (255, 0xc0 | TEXT, 2),
        (256, 0xd0 | TEXT, 3),
        (65535, 0xd0 | TEXT, 3),
        (65536, 0xe0 | TEXT, 5),
    ] {
        let string = "x".repeat(len);
        let bytes = to_vec(string.as_str()).unwrap();
        assert_eq!(bytes[0], header, "payload of {len} bytes");
        assert_eq!(bytes.len(), len + header_len, "payload of {len} bytes");
        assert_eq!(from_slice::<String>(&bytes).unwrap(), string);
    }

    // The sizes themselves are big-endian.
    assert_eq!(&to_vec("x".repeat(12).as_str()).unwrap()[..2], &[0xc7, 12]);
    assert_eq!(
        &to_vec("x".repeat(258).as_str()).unwrap()[..3],
        &[0xd7, 0x01, 0x02]
    );
    assert_eq!(
        &to_vec("x".repeat(65538).as_str()).unwrap()[..5],
        &[0xe7, 0x00, 0x01, 0x00, 0x02]
    );
}

#[test]
fn integer_extremes() {
    macro_rules! check {
        ($($ty:ty),* $(,)?) => {
            $(
                for value in [<$ty>::MIN, 0 as $ty, 1 as $ty, <$ty>::MAX] {
                    let bytes = to_vec(&value).unwrap();

                    assert_eq!(
                        from_slice::<$ty>(&bytes).unwrap(),
                        value,
                        "{}: {value}",
                        stringify!($ty)
                    );
                }
            )*
        }
    }

    check!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
    );
}

/// Decoding checks that the number in the payload fits the type it is being
/// decoded into.
#[test]
fn integer_bounds_are_checked() {
    assert_eq!(from_slice::<u8>(b"\x33255").unwrap(), 255);
    assert!(from_slice::<u8>(b"\x33256").is_err());
    assert_eq!(from_slice::<i8>(b"\x43-128").unwrap(), -128);
    assert!(from_slice::<i8>(b"\x33128").is_err());
    assert!(from_slice::<u32>(b"\x23-1").is_err());

    // A float payload is not an integer, however round it looks.
    assert!(from_slice::<u32>(b"\x351.0").is_err());
    // Nor is a string.
    assert!(from_slice::<u32>(b"\x171").is_err());
}

/// Floats survive the trip through their decimal spelling exactly, which is
/// what makes storing them as text viable.
#[test]
fn floats_roundtrip_bit_exactly() {
    for value in [
        0.0f64,
        -0.0,
        1.0,
        -1.0,
        0.1,
        1.0 / 3.0,
        1e-300,
        1e300,
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        // The smallest subnormal.
        5e-324,
    ] {
        let bytes = to_vec(&value).unwrap();

        assert_eq!(
            from_slice::<f64>(&bytes).unwrap().to_bits(),
            value.to_bits(),
            "f64: {value:e}"
        );
    }

    for value in [
        0.0f32,
        -0.0,
        1.0,
        -1.0,
        0.1,
        1.0 / 3.0,
        f32::MIN,
        f32::MAX,
        f32::MIN_POSITIVE,
        f32::EPSILON,
        1e-45,
    ] {
        let bytes = to_vec(&value).unwrap();

        assert_eq!(
            from_slice::<f32>(&bytes).unwrap().to_bits(),
            value.to_bits(),
            "f32: {value:e}"
        );
    }
}

#[test]
fn chars() {
    for c in ['a', 'ä', '→', '😀', '"', '\\', '\n', '\u{0}', '\u{7f}'] {
        let bytes = to_vec(&c).unwrap();
        assert_eq!(from_slice::<char>(&bytes).unwrap(), c, "{c:?}");
    }

    // A string which is not exactly one character is not a char.
    assert!(from_slice::<char>(b"\x27ab").is_err());
    assert!(from_slice::<char>(b"\x07").is_err());
}

/// Which of the two unescaped string types is used comes down to whether the
/// string would have to be escaped to be rendered as JSON.
#[test]
fn text_or_textraw_is_decided_by_escaping() {
    for string in ["", "plain", "\u{20}", "\u{7f}", "ä", "→", "😀", "'", "/"] {
        assert_eq!(
            to_vec(string).unwrap()[0] & 0x0f,
            TEXT,
            "{string:?} needs no escaping"
        );
    }

    for string in ["\"", "\\", "\n", "\t", "\u{0}", "\u{1f}", "a\u{1}b"] {
        assert_eq!(
            to_vec(string).unwrap()[0] & 0x0f,
            TEXTRAW,
            "{string:?} has to be escaped"
        );
    }

    // Either way the payload is the string itself, with no delimiters.
    assert_eq!(&to_vec("a\"b").unwrap()[1..], b"a\"b");
    assert_eq!(&to_vec("ab").unwrap()[1..], b"ab");
}

/// Strings are borrowed straight out of the input unless they had to be
/// unescaped first.
#[test]
fn strings_are_borrowed() {
    let bytes = to_vec("borrowed").unwrap();
    let string = from_slice::<&str>(&bytes).unwrap();
    assert_eq!(string, "borrowed");
    assert!(core::ptr::eq(string.as_ptr(), bytes[1..].as_ptr()));

    // The verbatim string type can be borrowed too.
    let bytes = to_vec("a\"b").unwrap();
    let string = from_slice::<&str>(&bytes).unwrap();
    assert_eq!(string, "a\"b");
    assert!(core::ptr::eq(string.as_ptr(), bytes[1..].as_ptr()));

    // An escaped string has to be translated through a scratch buffer, so there
    // is nothing in the input to borrow.
    assert!(from_slice::<&str>(b"\x48a\\nb").is_err());
    assert_eq!(from_slice::<String>(b"\x48a\\nb").unwrap(), "a\nb");
}

/// Decoding from a `&mut &[u8]` advances the slice, so several documents can be
/// read from the same buffer one after another.
#[test]
fn decoding_advances_a_mutable_slice() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&to_vec(&1u32).unwrap());
    bytes.extend_from_slice(&to_vec("two").unwrap());
    bytes.extend_from_slice(&to_vec(&vec![3u32]).unwrap());

    let mut slice = &bytes[..];
    assert_eq!(ENCODING.decode::<_, u32>(&mut slice).unwrap(), 1);
    assert_eq!(ENCODING.decode::<_, String>(&mut slice).unwrap(), "two");
    assert_eq!(ENCODING.decode::<_, Vec<u32>>(&mut slice).unwrap(), vec![3]);
    assert!(slice.is_empty());
}

#[test]
fn writing_to_slices() {
    const EXPECTED: &[u8] = &[0x4b, 0x13, b'1', 0x13, b'2'];

    let mut buf = [0; 64];
    let written = super::to_slice(&mut buf[..], &vec![1u32, 2]).unwrap();
    assert_eq!(&buf[..written], EXPECTED);

    // A slice which is too small is an error, not a truncated document.
    assert!(super::to_slice(&mut [0; 2][..], &vec![1u32, 2]).is_err());

    let bytes = super::to_fixed_bytes::<16, _>(&vec![1u32, 2]).unwrap();
    assert_eq!(bytes.as_slice(), EXPECTED);
    assert!(super::to_fixed_bytes::<2, _>(&vec![1u32, 2]).is_err());
}

#[cfg(feature = "std")]
#[test]
fn writing_to_a_writer() {
    let mut out = Vec::new();
    super::to_writer(&mut out, &vec![1u32, 2]).unwrap();
    assert_eq!(out, [0x4b, 0x13, b'1', 0x13, b'2']);
}

/// A document whose shape is not known ahead of time round-trips through the
/// dynamic value type, which is what being self-describing buys.
#[test]
fn value_roundtrip() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, name_all = "name")]
    struct Sample {
        int: i64,
        float: f64,
        string: String,
        escaped: String,
        list: Vec<u32>,
        nested: Vec<Vec<String>>,
        unit: (),
        flag: bool,
    }

    let expected = Sample {
        int: -42,
        float: 2.5,
        string: "plain".to_string(),
        escaped: "a\"b\nc".to_string(),
        list: vec![1, 2, 3],
        nested: vec![vec!["a".to_string()], vec![]],
        unit: (),
        flag: true,
    };

    let bytes = ENCODING.to_vec(&expected).unwrap();

    let value: Value<Global> = ENCODING.from_slice(&bytes).unwrap();
    let again = ENCODING.to_vec(&value).unwrap();

    // Re-encoding the buffered value reproduces the document exactly.
    assert_eq!(bytes, again);
    assert_eq!(ENCODING.from_slice::<Sample>(&again).unwrap(), expected);
}

/// The binary mode names fields by number rather than by name, and a number is
/// not something JSONB can use as an object key, so it is spelled out.
#[test]
fn binary_mode_numeric_names() {
    const BINARY: Encoding<Binary> = Encoding::new().with_mode();

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate)]
    struct Fields {
        first: u32,
        second: u32,
    }

    let value = Fields {
        first: 1,
        second: 2,
    };
    let bytes = BINARY.to_vec(&value).unwrap();

    assert_eq!(bytes, *b"\x8c\x170\x131\x171\x132".as_slice());
    assert_eq!(BINARY.from_slice::<Fields>(&bytes).unwrap(), value);

    // Which means it reads back as an ordinary object with string keys.
    assert_eq!(
        BINARY.from_slice::<BTreeMap<String, u32>>(&bytes).unwrap(),
        BTreeMap::from([("0".to_string(), 1), ("1".to_string(), 2)])
    );
}

/// A map keyed by numbers goes the same way, and the keys parse back out of the
/// strings they were spelled as.
#[test]
fn maps_with_numeric_keys() {
    let map = BTreeMap::from([(1u32, "a".to_string()), (2, "b".to_string())]);
    let bytes = ENCODING.to_vec(&map).unwrap();

    assert_eq!(bytes, *b"\x8c\x171\x17a\x172\x17b".as_slice());
    assert_eq!(
        ENCODING
            .from_slice::<BTreeMap<u32, String>>(&bytes)
            .unwrap(),
        map
    );
    assert_eq!(
        ENCODING
            .from_slice::<BTreeMap<String, String>>(&bytes)
            .unwrap(),
        BTreeMap::from([
            ("1".to_string(), "a".to_string()),
            ("2".to_string(), "b".to_string())
        ])
    );

    // A key which is not a string is not a key.
    assert!(from_slice::<BTreeMap<u32, u32>>(&container(OBJECT, b"\x131\x132")).is_err());
}

/// A packed struct has no field names at all, so it is written as an array.
#[test]
fn packed_structs() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, packed)]
    struct Packed {
        a: u32,
        b: bool,
        c: String,
    }

    let value = Packed {
        a: 1,
        b: true,
        c: "x".to_string(),
    };

    let bytes = ENCODING.to_vec(&value).unwrap();
    assert_eq!(bytes, *b"\x5b\x131\x01\x17x".as_slice());
    assert_eq!(ENCODING.from_slice::<Packed>(&bytes).unwrap(), value);
}

/// Byte strings have no JSON representation, so like the JSON encoder they are
/// written as arrays of numbers.
#[test]
fn bytes_are_arrays_of_numbers() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, packed)]
    struct Container {
        #[musli(bytes)]
        bytes: Vec<u8>,
    }

    let value = Container {
        bytes: vec![1, 2, 30],
    };

    let bytes = ENCODING.to_vec(&value).unwrap();
    assert_eq!(bytes, *b"\x8b\x7b\x131\x132\x2330".as_slice());
    assert_eq!(ENCODING.from_slice::<Container>(&bytes).unwrap(), value);

    // Fixed size arrays go the same way, and their length is checked.
    let bytes = ENCODING.to_vec(&[1u8, 2, 3]).unwrap();
    assert_eq!(ENCODING.from_slice::<[u8; 3]>(&bytes).unwrap(), [1, 2, 3]);
    assert!(ENCODING.from_slice::<[u8; 2]>(&bytes).is_err());
    assert!(ENCODING.from_slice::<[u8; 4]>(&bytes).is_err());
}

/// Skipping an unknown field never looks at its contents, so it costs the same
/// whatever is in there.
#[test]
fn skipping_does_not_depend_on_depth() {
    #[derive(Debug, PartialEq, Decode)]
    #[musli(crate, name_all = "name")]
    struct Partial {
        c: String,
    }

    let mut deep = to_vec(&Vec::<u32>::new()).unwrap();

    for _ in 0..1000 {
        deep = container(ARRAY, &deep);
    }

    let mut payload = to_vec("deep").unwrap();
    payload.extend_from_slice(&deep);
    payload.extend_from_slice(&to_vec("c").unwrap());
    payload.extend_from_slice(&to_vec("hello").unwrap());

    let bytes = container(OBJECT, &payload);

    assert_eq!(
        ENCODING.from_slice::<Partial>(&bytes).unwrap(),
        Partial {
            c: "hello".to_string()
        }
    );
}

/// Every kind of element can be skipped over, including the ones musli never
/// writes itself.
#[test]
fn every_element_type_can_be_skipped() {
    #[derive(Debug, PartialEq, Decode)]
    #[musli(crate, name_all = "name")]
    struct Partial {
        keep: u32,
    }

    let elements: &[&[u8]] = &[
        b"\x00",           // NULL
        b"\x01",           // TRUE
        b"\x02",           // FALSE
        b"\x13\x31",       // INT
        b"\x440x1f",       // INT5
        b"\x351.5",        // FLOAT
        b"\x26.5",         // FLOAT5
        b"\x37abc",        // TEXT
        b"\x48a\\nb",      // TEXTJ
        b"\x49a\\qb",      // TEXT5
        b"\x3aa\"b",       // TEXTRAW
        b"\x2b\x131",      // ARRAY
        b"\x4c\x17a\x131", // OBJECT
    ];

    for (index, element) in elements.iter().enumerate() {
        let mut payload = to_vec("skipped").unwrap();
        payload.extend_from_slice(element);
        payload.extend_from_slice(&to_vec("keep").unwrap());
        payload.extend_from_slice(&to_vec(&7u32).unwrap());

        let bytes = container(OBJECT, &payload);

        assert_eq!(
            ENCODING.from_slice::<Partial>(&bytes).unwrap(),
            Partial { keep: 7 },
            "element {index}"
        );
    }
}

/// Malformed input is always reported as an error, and never as a panic.
#[test]
fn malformed_input() {
    let cases: &[(&str, &[u8])] = &[
        ("empty input", b""),
        ("truncated one byte size", b"\xc7"),
        ("truncated two byte size", b"\xd7\x01"),
        ("truncated four byte size", b"\xe7\x00\x00"),
        ("truncated eight byte size", b"\xf7\x00"),
        ("reserved element type 13", b"\x0d"),
        ("reserved element type 14", b"\x0e"),
        ("reserved element type 15", b"\x0f"),
        ("payload past the end of the input", b"\x97abc"),
        ("string which is not utf-8", &[0x17, 0xff]),
        ("array element past the end", b"\x2b\x97a"),
        ("object with a key but no value", b"\x2c\x17a"),
        ("object with a key which is not a string", b"\x2c\x131"),
        ("dangling escape", b"\x18\\"),
        ("truncated unicode escape", b"\x48\\u00"),
        ("bad hex in a unicode escape", b"\x68\\u00zz"),
        ("lone high surrogate", b"\x68\\ud83d"),
        (
            "high surrogate followed by a plain escape",
            b"\xc8\x08\\ud83d\\n",
        ),
        ("integer payload which is not a number", b"\x13a"),
        ("float payload which is not a number", b"\x35abc"),
        ("trailing input", b"\x131\x01"),
    ];

    for (name, bytes) in cases {
        // Once without knowing what is being decoded, which walks the document
        // by its element types.
        assert!(
            from_slice::<Value<Global>>(bytes).is_err(),
            "{name}: expected an error decoding into a value"
        );

        // And once with a type in mind, which takes a different path through
        // the decoder for everything but the containers.
        assert!(
            from_slice::<String>(bytes).is_err() || from_slice::<u32>(bytes).is_err(),
            "{name}: expected an error decoding into a type"
        );
    }
}

/// Documents which are valid but empty, which are easy to get wrong at the
/// boundaries.
#[test]
fn empty_documents() {
    assert_eq!(to_vec(&Vec::<u32>::new()).unwrap(), [ARRAY]);
    assert_eq!(to_vec(&BTreeMap::<String, u32>::new()).unwrap(), [OBJECT]);
    assert_eq!(to_vec("").unwrap(), [TEXT]);

    assert!(from_slice::<Vec<u32>>(&[ARRAY]).unwrap().is_empty());
    assert!(
        from_slice::<BTreeMap<String, u32>>(&[OBJECT])
            .unwrap()
            .is_empty()
    );
    assert_eq!(from_slice::<String>(&[TEXT]).unwrap(), "");

    // An empty document is not a document.
    assert!(from_slice::<Vec<u32>>(b"").is_err());
}

/// Randomly generated documents survive the round trip.
#[test]
fn random_roundtrip() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, name_all = "name")]
    struct Sample {
        string: String,
        signed: i64,
        unsigned: u64,
        float: f64,
        nested: Vec<Vec<String>>,
        map: BTreeMap<String, i32>,
    }

    let mut rng = Rng(0x9e3779b97f4a7c15);

    for _ in 0..2000 {
        let expected = Sample {
            string: rng.string(24),
            signed: rng.next() as i64,
            unsigned: rng.next(),
            float: f64::from_bits(rng.next()),
            nested: (0..rng.below(4))
                .map(|_| (0..rng.below(4)).map(|_| rng.string(8)).collect())
                .collect(),
            map: (0..rng.below(4))
                .map(|_| (rng.string(4), rng.next() as i32))
                .collect(),
        };

        // A NaN never compares equal to itself, so leave those to `floats`.
        if expected.float.is_nan() {
            continue;
        }

        let bytes = ENCODING.to_vec(&expected).unwrap();
        let actual: Sample = ENCODING.from_slice(&bytes).unwrap();
        assert_eq!(actual, expected);

        // And the same document read without knowing its shape.
        let value: Value<Global> = ENCODING.from_slice(&bytes).unwrap();
        assert_eq!(ENCODING.to_vec(&value).unwrap(), bytes);
    }
}

/// A small xorshift generator, so that the randomized tests do not need a
/// dependency and always run the same sequence.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// A string of up to `max` characters drawn from a set which covers every
    /// UTF-8 length as well as everything which has to be escaped.
    fn string(&mut self, max: usize) -> String {
        const ALPHABET: &[char] = &[
            'a', 'z', '0', ' ', '"', '\\', '/', '\n', '\t', '\u{0}', '\u{1f}', '\u{7f}', 'ä', '→',
            '😀',
        ];

        (0..self.below(max))
            .map(|_| ALPHABET[self.below(ALPHABET.len())])
            .collect()
    }
}
