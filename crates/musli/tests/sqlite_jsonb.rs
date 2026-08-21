//! Interoperability with SQLite, which is what the JSONB format exists for.
//!
//! Every document in this file is handed to a real in-memory SQLite database,
//! either to be rendered as JSON or to be produced by SQLite so that it can be
//! decoded again.
//!
//! Miri cannot call into SQLite's C library, so these are skipped there.

#![cfg(all(
    feature = "std",
    feature = "alloc",
    feature = "sqlite-jsonb",
    not(miri)
))]

use std::collections::BTreeMap;

use musli::alloc::Global;
use musli::mode::Text;
use musli::value::Value;
use musli::{Decode, Encode, sqlite_jsonb};
use sqll::Connection;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "name")]
struct Person {
    name: String,
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "name")]
struct Outer {
    a: Inner,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_all = "name")]
struct Inner {
    b: Vec<u32>,
}

/// A connection together with the helpers used to talk to SQLite about JSONB.
struct Sqlite {
    c: Connection,
}

impl Sqlite {
    fn new() -> Self {
        Self {
            c: Connection::open_in_memory().expect("Failed to open database"),
        }
    }

    /// Render a JSONB blob as JSON text, the way `json_extract` and friends
    /// would see it.
    #[track_caller]
    fn json(&self, blob: &[u8]) -> String {
        let mut stmt = self.c.prepare("SELECT json(?)").expect("Failed to prepare");
        stmt.bind(blob).expect("Failed to bind blob");
        stmt.next::<String>()
            .expect("Failed to render blob as JSON")
            .expect("Query returned no row")
    }

    /// Test that a blob is a valid RFC 8259 JSONB document.
    #[track_caller]
    fn is_valid(&self, blob: &[u8]) -> bool {
        let mut stmt = self
            .c
            .prepare("SELECT json_valid(?, 8)")
            .expect("Failed to prepare");
        stmt.bind(blob).expect("Failed to bind blob");
        stmt.next::<i64>().expect("Failed to validate blob") == Some(1)
    }

    /// Have SQLite translate JSON text into a JSONB blob.
    #[track_caller]
    fn jsonb(&self, json: &str) -> Vec<u8> {
        let mut stmt = self
            .c
            .prepare("SELECT jsonb(?)")
            .expect("Failed to prepare");
        stmt.bind(json).expect("Failed to bind text");
        stmt.next::<Vec<u8>>()
            .expect("Failed to translate JSON to JSONB")
            .expect("Query returned no row")
    }

    /// Decode what SQLite produces for the given JSON document.
    #[track_caller]
    fn decode<T>(&self, json: &str) -> T
    where
        T: for<'de> Decode<'de, Text, Global>,
    {
        let blob = self.jsonb(json);
        sqlite_jsonb::from_slice(&blob).expect("Failed to decode blob written by SQLite")
    }

    /// Evaluate a query which takes a single blob and returns a single blob.
    #[track_caller]
    fn blob(&self, sql: &str, blob: &[u8]) -> Vec<u8> {
        let mut stmt = self.c.prepare(sql).expect("Failed to prepare");
        stmt.bind(blob).expect("Failed to bind blob");
        stmt.next::<Vec<u8>>()
            .expect("Failed to evaluate query")
            .expect("Query returned no row")
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

    /// A string drawn from a set which covers every UTF-8 length as well as
    /// everything which has to be escaped to be rendered as JSON.
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

/// What musli encodes is a valid JSONB document which SQLite renders as the
/// JSON it was meant to be.
#[test]
fn sqlite_renders_what_musli_encodes() {
    let db = Sqlite::new();

    macro_rules! check {
        ($value:expr, $json:expr) => {{
            let blob = sqlite_jsonb::to_vec(&$value).unwrap();
            assert!(
                db.is_valid(&blob),
                "{}: not a valid JSONB document",
                stringify!($value)
            );
            assert_eq!(db.json(&blob), $json, "{}", stringify!($value));
        }};
    }

    check!((), "null");
    check!(Option::<u32>::None, "null");
    check!(true, "true");
    check!(false, "false");
    check!(1u32, "1");
    check!(-1i32, "-1");
    check!(u64::MAX, "18446744073709551615");
    check!(i64::MIN, "-9223372036854775808");
    check!(0.5f64, "0.5");
    check!(1.0f64, "1.0");
    check!(-0.0f64, "-0.0");
    check!(1.5e10f64, "15000000000.0");
    // The infinities are spelled the way SQLite spells them.
    check!(f64::INFINITY, "9e999");
    check!(f64::NEG_INFINITY, "-9e999");
    check!("abc", "\"abc\"");
    check!("héllo → ✓", "\"héllo → ✓\"");
    check!(Vec::<u32>::new(), "[]");
    check!(vec![1u32, 2], "[1,2]");
    check!(vec![vec![1u32], vec![], vec![2, 3]], "[[1],[],[2,3]]");
    check!(vec![1u8, 2, 3], "[1,2,3]");
    check!(Some(vec![Some("a".to_string()), None]), "[\"a\",null]");
    check!(
        Person {
            name: "Bob".to_string(),
            age: 61,
        },
        r#"{"name":"Bob","age":61}"#
    );
    check!(
        Outer {
            a: Inner { b: vec![1, 2, 3] }
        },
        r#"{"a":{"b":[1,2,3]}}"#
    );
    // Strings which need escaping are stored verbatim, and SQLite escapes them
    // again when it renders them.
    check!("a\"b", r#""a\"b""#);
    check!("a\nb", r#""a\nb""#);
    check!("a\\b", r#""a\\b""#);
}

/// A JSONB document written by musli renders as exactly the same JSON as the
/// same value encoded with the JSON encoder.
#[test]
fn agrees_with_the_json_encoder() {
    let db = Sqlite::new();

    macro_rules! check {
        ($value:expr) => {{
            let blob = sqlite_jsonb::to_vec(&$value).unwrap();
            assert_eq!(
                db.json(&blob),
                musli::json::to_string(&$value).unwrap(),
                "{}",
                stringify!($value)
            );
        }};
    }

    check!(1u32);
    check!(-1i32);
    check!(0.5f64);
    check!("a\"b\nc\\d");
    check!("héllo → ✓");
    check!(vec![vec![1u32], vec![], vec![2, 3]]);
    check!(Person {
        name: "Bob".to_string(),
        age: 61,
    });
    check!(Outer {
        a: Inner { b: vec![1, 2, 3] }
    });
}

/// For the documents which have only one canonical encoding, musli writes
/// exactly the same bytes SQLite does.
#[test]
fn encodes_the_same_bytes_as_sqlite() {
    let db = Sqlite::new();

    macro_rules! check {
        ($value:expr, $json:expr) => {
            assert_eq!(
                sqlite_jsonb::to_vec(&$value).unwrap(),
                db.jsonb($json),
                "{}",
                stringify!($value)
            );
        };
    }

    check!((), "null");
    check!(true, "true");
    check!(false, "false");
    check!(1u32, "1");
    check!(-0.5f64, "-0.5");
    check!(f64::INFINITY, "Infinity");
    check!(f64::NEG_INFINITY, "-Infinity");
    check!("plain", "\"plain\"");
    check!(Vec::<u32>::new(), "[]");
    check!(vec![1u32, 2, 3], "[1,2,3]");
    check!(BTreeMap::<String, u32>::new(), "{}");
    check!(
        Person {
            name: "Bob".to_string(),
            age: 61,
        },
        r#"{"name":"Bob","age":61}"#
    );
}

/// Documents written by SQLite decode back into the values they describe.
#[test]
fn decodes_what_sqlite_writes() {
    let db = Sqlite::new();

    assert_eq!(db.decode::<Option<u32>>("null"), None);
    assert!(db.decode::<bool>("true"));
    assert!(!db.decode::<bool>("false"));
    assert_eq!(db.decode::<u64>("18446744073709551615"), u64::MAX);
    assert_eq!(db.decode::<i64>("-9223372036854775808"), i64::MIN);
    assert_eq!(db.decode::<f64>("1e3"), 1000.0);
    assert_eq!(db.decode::<f64>("-0.5"), -0.5);
    assert_eq!(db.decode::<f64>("Infinity"), f64::INFINITY);
    assert_eq!(db.decode::<f64>("-Infinity"), f64::NEG_INFINITY);
    assert_eq!(db.decode::<Vec<u32>>("[]"), Vec::<u32>::new());
    assert_eq!(db.decode::<Vec<u32>>("[1,2,3]"), vec![1, 2, 3]);
    assert_eq!(
        db.decode::<(u32, f64, bool, bool, Option<u32>)>("[1,2.5,true,false,null]"),
        (1, 2.5, true, false, None)
    );
    assert_eq!(
        db.decode::<Person>(r#"{"name":"Bob","age":61}"#),
        Person {
            name: "Bob".to_string(),
            age: 61,
        }
    );
    assert_eq!(
        db.decode::<Outer>(r#"{"a":{"b":[1,2,3]}}"#),
        Outer {
            a: Inner { b: vec![1, 2, 3] }
        }
    );
    assert!(db.decode::<BTreeMap<String, u32>>("{}").is_empty());

    // JSON5, which SQLite also accepts as input. These produce the `INT5`,
    // `FLOAT5` and `TEXT5` element types which musli never writes itself.
    assert_eq!(db.decode::<u32>("0x1f"), 31);
    assert_eq!(db.decode::<i32>("-0x1f"), -31);
    assert_eq!(db.decode::<f64>(".5"), 0.5);
    assert_eq!(db.decode::<f64>("5."), 5.0);
    assert_eq!(
        db.decode::<BTreeMap<String, u32>>("{a:1}"),
        BTreeMap::from([("a".to_string(), 1)])
    );
    assert_eq!(db.decode::<Vec<String>>(r"['a\x41b']"), vec!["aAb"]);
}

/// SQLite writes strings which contain escapes as `TEXTJ`, and JSON5 documents
/// can additionally contain `TEXT5`. Neither is written by musli, but both are
/// translated when decoding.
#[test]
fn decodes_escaped_strings_written_by_sqlite() {
    let db = Sqlite::new();

    for string in [
        "plain",
        "a\nb",
        "tab\there",
        "a\"b",
        "a\\b",
        "héllo → ✓",
        "😀",
        "\u{7f}",
    ] {
        let json = musli::json::to_string(string).unwrap();

        assert_eq!(
            db.decode::<String>(&json),
            string,
            "{string:?} does not survive SQLite"
        );
    }

    // Spelled out as escapes rather than as the characters themselves, which is
    // what makes SQLite write `TEXTJ`.
    assert_eq!(db.decode::<String>(r#""é""#), "é");
    assert_eq!(db.decode::<String>(r#""😀""#), "😀");
    assert_eq!(db.decode::<String>(r#""\/\b\f""#), "/\u{8}\u{c}");
}

/// A document whose shape is not known ahead of time can be decoded into a
/// dynamic [`Value`], which is what the self-describing part of the format is
/// for.
#[test]
fn decodes_unknown_shapes() {
    let db = Sqlite::new();

    for json in [
        r#"{"a":{"b":[1,2,{"c":"x"}]}}"#,
        r#"[1,-2,3.5,true,false,null,"s",[],{}]"#,
        r#"{"nested":{"deeply":{"list":[{"k":"v"}]}}}"#,
    ] {
        let blob = db.jsonb(json);
        let value: Value<Global> = sqlite_jsonb::from_slice(&blob).unwrap();
        assert_eq!(musli::json::to_string(&value).unwrap(), json);
    }
}

/// Blobs written by musli go into a table, are queried by SQL, and come back
/// out as the same bytes.
#[test]
fn roundtrips_through_a_table() {
    let db = Sqlite::new();

    db.c.execute("CREATE TABLE packages (id INTEGER PRIMARY KEY, doc BLOB NOT NULL)")
        .unwrap();

    let people = [
        Person {
            name: "Aristotle".to_string(),
            age: 61,
        },
        Person {
            name: "Plato".to_string(),
            age: 80,
        },
        Person {
            // Something which has to be stored verbatim.
            name: "He said \"hi\"\n".to_string(),
            age: 1,
        },
    ];

    let mut insert =
        db.c.prepare("INSERT INTO packages (doc) VALUES (?)")
            .unwrap();

    for person in &people {
        insert
            .execute(&sqlite_jsonb::to_vec(person).unwrap()[..])
            .unwrap();
        insert.reset().unwrap();
    }

    // SQLite reads the documents with the ordinary JSON operators.
    let mut stmt =
        db.c.prepare("SELECT doc ->> '$.name', doc ->> '$.age' FROM packages ORDER BY id")
            .unwrap();

    let mut rows = Vec::new();

    while let Some(row) = stmt.next::<(String, i64)>().unwrap() {
        rows.push(row);
    }

    assert_eq!(
        rows,
        people
            .iter()
            .map(|p| (p.name.clone(), i64::from(p.age)))
            .collect::<Vec<_>>()
    );

    // And the blobs decode again without SQLite ever rendering them as text.
    let mut stmt =
        db.c.prepare("SELECT doc FROM packages ORDER BY id")
            .unwrap();

    let mut decoded = Vec::new();

    while let Some(doc) = stmt.next::<&[u8]>().unwrap() {
        decoded.push(sqlite_jsonb::from_slice::<Person>(doc).unwrap());
    }

    assert_eq!(decoded, people);
}

/// `jsonb_extract` hands back a JSONB blob of its own, so a nested document can
/// be pulled out by SQLite and decoded on its own.
#[test]
fn decodes_sub_documents_extracted_by_sqlite() {
    let db = Sqlite::new();

    let outer = Outer {
        a: Inner { b: vec![1, 2, 3] },
    };

    let blob = sqlite_jsonb::to_vec(&outer).unwrap();

    let extracted = db.blob("SELECT jsonb_extract(?, '$.a')", &blob);
    assert_eq!(
        sqlite_jsonb::from_slice::<Inner>(&extracted).unwrap(),
        outer.a
    );

    let extracted = db.blob("SELECT jsonb_extract(?, '$.a.b')", &blob);
    assert_eq!(
        sqlite_jsonb::from_slice::<Vec<u32>>(&extracted).unwrap(),
        vec![1, 2, 3]
    );

    // A path which lands on a scalar is handed back as an ordinary SQL value
    // rather than as a document.
    let mut stmt = db.c.prepare("SELECT ? ->> '$.a.b[1]'").unwrap();
    stmt.bind(&blob[..]).unwrap();
    assert_eq!(stmt.next::<i64>().unwrap(), Some(2));
}

/// Documents which SQLite builds out of a musli-written one are still
/// decodable, which is what makes the blobs safe to edit in SQL.
#[test]
fn decodes_documents_edited_by_sqlite() {
    let db = Sqlite::new();

    let blob = sqlite_jsonb::to_vec(&Person {
        name: "Aristotle".to_string(),
        age: 61,
    })
    .unwrap();

    // Adding a field. SQL text values become the verbatim string type, which is
    // the same one musli writes for strings which have to be escaped.
    let edited = db.blob("SELECT jsonb_set(?, '$.city', 'Stagira')", &blob);

    #[derive(Debug, PartialEq, Decode)]
    #[musli(name_all = "name")]
    struct Citizen {
        name: String,
        age: u32,
        city: String,
    }

    assert_eq!(
        sqlite_jsonb::from_slice::<Citizen>(&edited).unwrap(),
        Citizen {
            name: "Aristotle".to_string(),
            age: 61,
            city: "Stagira".to_string(),
        }
    );

    // Removing one, which the narrower type does not miss.
    let edited = db.blob("SELECT jsonb_remove(?, '$.age')", &blob);

    #[derive(Debug, PartialEq, Decode)]
    #[musli(name_all = "name")]
    struct Name {
        name: String,
    }

    assert_eq!(
        sqlite_jsonb::from_slice::<Name>(&edited).unwrap(),
        Name {
            name: "Aristotle".to_string()
        }
    );

    // And merging a patch into it.
    let edited = db.blob(r#"SELECT jsonb_patch(?, '{"age":62}')"#, &blob);

    assert_eq!(
        sqlite_jsonb::from_slice::<Person>(&edited).unwrap(),
        Person {
            name: "Aristotle".to_string(),
            age: 62,
        }
    );
}

/// The aggregate builders produce whole documents, which decode like any other.
#[test]
fn decodes_documents_aggregated_by_sqlite() {
    let db = Sqlite::new();

    db.c.execute(
        r#"
            CREATE TABLE numbers (name TEXT, value INTEGER);
            INSERT INTO numbers VALUES ('one', 1), ('two', 2), ('three', 3);
            "#,
    )
    .unwrap();

    let mut stmt =
        db.c.prepare("SELECT jsonb_group_array(value) FROM numbers")
            .unwrap();
    let doc = stmt.next::<Vec<u8>>().unwrap().unwrap();
    assert_eq!(
        sqlite_jsonb::from_slice::<Vec<u32>>(&doc).unwrap(),
        vec![1, 2, 3]
    );

    let mut stmt =
        db.c.prepare("SELECT jsonb_group_object(name, value) FROM numbers")
            .unwrap();
    let doc = stmt.next::<Vec<u8>>().unwrap().unwrap();
    assert_eq!(
        sqlite_jsonb::from_slice::<BTreeMap<String, u32>>(&doc).unwrap(),
        BTreeMap::from([
            ("one".to_string(), 1),
            ("two".to_string(), 2),
            ("three".to_string(), 3),
        ])
    );
}

/// Documents large enough to need the two and four byte payload sizes, in both
/// directions.
#[test]
fn large_documents() {
    let db = Sqlite::new();

    for len in [300usize, 70_000] {
        let string = "x".repeat(len);

        // Written by musli, read by SQLite.
        let blob = sqlite_jsonb::to_vec(string.as_str()).unwrap();
        assert!(db.is_valid(&blob), "{len} bytes");
        assert_eq!(db.json(&blob), format!("\"{string}\""), "{len} bytes");

        // Written by SQLite, read by musli.
        assert_eq!(
            db.decode::<String>(&musli::json::to_string(string.as_str()).unwrap()),
            string,
            "{len} bytes"
        );
    }

    // And an array with enough elements to push the payload past both bounds.
    for count in [100usize, 20_000] {
        let values = (0..count as u32).collect::<Vec<_>>();
        let blob = sqlite_jsonb::to_vec(&values).unwrap();

        assert!(db.is_valid(&blob), "{count} elements");

        // SQLite can still reach into the document by path, which means it
        // walked every element header to get there.
        let mut stmt =
            db.c.prepare("SELECT ? ->> '$[1]', json_array_length(?1)")
                .unwrap();
        stmt.bind(&blob[..]).unwrap();
        assert_eq!(stmt.next::<(i64, i64)>().unwrap(), Some((1, count as i64)));
    }
}

/// Nesting is bounded by the payload sizes rather than by anything the format
/// says, so it goes as deep as the documents do.
#[test]
fn deeply_nested_documents() {
    let db = Sqlite::new();

    const DEPTH: usize = 30;

    let json = format!("{}1{}", "[".repeat(DEPTH), "]".repeat(DEPTH));

    // Written by SQLite, read by musli.
    let blob = db.jsonb(&json);
    let value: Value<Global> = sqlite_jsonb::from_slice(&blob).unwrap();
    assert_eq!(musli::json::to_string(&value).unwrap(), json);

    // Written by musli, read by SQLite. Re-encoding the value which was decoded
    // out of SQLite's document reproduces it byte for byte.
    assert_eq!(sqlite_jsonb::to_vec(&value).unwrap(), blob);
    assert!(db.is_valid(&blob));
}

/// Randomly generated documents are always valid JSONB, and SQLite renders them
/// as exactly what the JSON encoder would have written.
#[test]
fn random_documents_agree_with_sqlite() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(name_all = "name")]
    struct Sample {
        string: String,
        signed: i64,
        unsigned: u64,
        float: f64,
        list: Vec<String>,
        nested: Vec<Vec<i32>>,
        map: BTreeMap<String, i32>,
    }

    let db = Sqlite::new();
    let mut rng = Rng(0xa076_1d64_78bd_642f);

    for _ in 0..500 {
        let value = Sample {
            string: rng.string(24),
            signed: rng.next() as i64,
            unsigned: rng.next(),
            float: f64::from_bits(rng.next()),
            list: (0..rng.below(4)).map(|_| rng.string(8)).collect(),
            nested: (0..rng.below(4))
                .map(|_| (0..rng.below(4)).map(|_| rng.next() as i32).collect())
                .collect(),
            map: (0..rng.below(4))
                .map(|_| (rng.string(4), rng.next() as i32))
                .collect(),
        };

        // The infinities and NaN have no canonical JSON spelling, so the two
        // encoders are only expected to agree on the finite ones.
        if !value.float.is_finite() {
            continue;
        }

        let blob = sqlite_jsonb::to_vec(&value).unwrap();

        assert!(db.is_valid(&blob), "{value:?}");
        assert_eq!(
            db.json(&blob),
            musli::json::to_string(&value).unwrap(),
            "{value:?}"
        );

        // And what SQLite makes of that JSON decodes back to the same value.
        assert_eq!(db.decode::<Sample>(&db.json(&blob)), value);
    }
}
