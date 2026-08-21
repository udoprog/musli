//! Storing Müsli-encoded documents in SQLite as JSONB.
//!
//! JSONB is what SQLite stores internally for its `json` functions, so a blob
//! written by [`musli::sqlite_jsonb`] can be put straight into a column and
//! queried with SQL. Nothing translates it on the way in or on the way out:
//! the same bytes are written by the encoder, indexed by SQLite and decoded
//! again.
//!
//! Run with:
//!
//! ```text
//! cargo run -p musli --example sqlite_jsonb --features sqlite-jsonb
//! ```

use std::error::Error;

use musli::sqlite_jsonb;
use musli::{Decode, Encode};
use sqll::Connection;

/// The document which is stored in the `doc` column of each row.
#[derive(Debug, Encode, Decode)]
struct Package {
    name: String,
    version: String,
    keywords: Vec<String>,
    metadata: Metadata,
}

#[derive(Debug, Encode, Decode)]
struct Metadata {
    downloads: u64,
    rating: Option<f64>,
    description: String,
}

/// A view over the same documents which only cares about two of the fields.
///
/// Decoding this skips over everything else without looking at its contents,
/// since every JSONB element knows the size of its payload.
#[derive(Debug, Decode)]
struct Name<'a> {
    name: &'a str,
    version: &'a str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let c = Connection::open_in_memory()?;

    c.execute(
        "CREATE TABLE packages (
            id INTEGER PRIMARY KEY,
            doc BLOB NOT NULL
        )",
    )?;

    // Encode with musli and hand the bytes to SQLite as an ordinary blob.
    let mut insert = c.prepare("INSERT INTO packages (doc) VALUES (?)")?;

    for package in packages() {
        insert.execute(sqlite_jsonb::to_vec(&package)?)?;
        insert.reset()?;
    }

    // The same table can just as well be filled by SQLite itself, which parses
    // JSON text into the very same representation.
    let mut insert = c.prepare("INSERT INTO packages (doc) VALUES (jsonb(?))")?;

    insert.execute(
        r#"{
            "name": "sqll",
            "version": "0.14.3",
            "keywords": ["database", "sqlite"],
            "metadata": {
                "downloads": 1200,
                "rating": null,
                "description": "Doesn't get in your \"way\""
            }
        }"#,
    )?;

    section("What SQLite sees");

    // SQLite reads the documents musli wrote with the ordinary JSON operators,
    // so paths, types and lengths all work without decoding anything in Rust.
    let mut stmt = c.prepare(
        "SELECT doc ->> '$.name',
                doc ->> '$.metadata.downloads',
                json_type(doc, '$.metadata.rating'),
                json_array_length(doc, '$.keywords')
         FROM packages
         ORDER BY doc ->> '$.metadata.downloads' DESC",
    )?;

    while let Some((name, downloads, rating, keywords)) =
        stmt.next::<(String, i64, String, i64)>()?
    {
        println!("{name}: {downloads} downloads, rating {rating}, {keywords} keywords");
    }

    section("Querying inside the documents");

    // Filtering happens in SQLite, over the blobs musli wrote.
    let mut stmt = c.prepare(
        "SELECT DISTINCT packages.doc ->> '$.name'
         FROM packages, json_each(packages.doc, '$.keywords')
         WHERE json_each.value = ?
         ORDER BY 1",
    )?;

    stmt.bind("serialization")?;

    while let Some(name) = stmt.next::<String>()? {
        println!("tagged `serialization`: {name}");
    }

    section("Decoding whole rows");

    // And the blobs come back out as blobs, so they decode without SQLite ever
    // rendering them as text.
    let mut stmt = c.prepare("SELECT doc FROM packages ORDER BY id")?;

    while let Some(doc) = stmt.next::<&[u8]>()? {
        let package: Package = sqlite_jsonb::from_slice(doc)?;

        println!(
            "{} {} ({} bytes): {}",
            package.name,
            package.version,
            doc.len(),
            package.metadata.description
        );
    }

    section("Decoding only what is needed");

    // The same rows decoded into a type which knows about two fields. The
    // keywords and the metadata are skipped over by their payload size alone.
    let mut stmt = c.prepare("SELECT doc FROM packages ORDER BY id")?;

    while let Some(doc) = stmt.next::<&[u8]>()? {
        let name: Name<'_> = sqlite_jsonb::from_slice(doc)?;
        println!("{} {}", name.name, name.version);
    }

    section("Decoding a sub-document");

    // `jsonb_extract` hands back a JSONB blob of its own, so a nested document
    // can be pulled out by SQLite and decoded on its own.
    let mut stmt = c.prepare(
        "SELECT jsonb_extract(doc, '$.metadata')
         FROM packages
         WHERE doc ->> '$.name' = ?",
    )?;

    stmt.bind("musli")?;

    if let Some(doc) = stmt.next::<&[u8]>()? {
        let metadata: Metadata = sqlite_jsonb::from_slice(doc)?;
        println!("{metadata:?}");
    }

    section("The bytes themselves");

    let blob = sqlite_jsonb::to_vec(&(1u32, "two", true))?;

    let mut stmt = c.prepare("SELECT json(?)")?;
    stmt.bind(&blob[..])?;

    println!("musli wrote: {}", hex(&blob));
    println!(
        "SQLite reads it as: {}",
        stmt.next::<String>()?.unwrap_or_default()
    );

    Ok(())
}

fn packages() -> Vec<Package> {
    vec![
        Package {
            name: String::from("musli"),
            version: String::from("0.1.5"),
            keywords: vec![String::from("serialization"), String::from("no_std")],
            metadata: Metadata {
                downloads: 42_000,
                rating: Some(4.5),
                description: String::from("A flexible and efficient serialization framework"),
            },
        },
        Package {
            name: String::from("musli-zerocopy"),
            version: String::from("0.1.5"),
            keywords: vec![String::from("serialization"), String::from("zerocopy")],
            metadata: Metadata {
                downloads: 7_500,
                rating: Some(4.8),
                // Quotes and newlines are stored verbatim, and SQLite escapes
                // them again whenever it renders the document as text.
                description: String::from("Zero copy \"primitives\"\nfor Müsli"),
            },
        },
    ]
}

fn section(title: &str) {
    println!();
    println!("== {title} ==");
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(String::new(), |mut s, b| {
        _ = write!(s, "{b:02x}");
        s
    })
}
