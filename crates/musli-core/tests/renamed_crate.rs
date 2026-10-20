//! A renamed `musli-core` dependency is named with `#[musli(crate = ..)]`.
//!
//! Cargo does not let a package depend on itself under another name, so the
//! rename is made with `extern crate`, which puts `musli_core01` in the extern
//! prelude the same way `musli-core01 = { package = "musli-core" }` would.

extern crate musli_core as musli_core01;

use musli_core01::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate = musli_core01)]
struct Person<'a> {
    name: &'a str,
    age: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate = musli_core01, Text, name_all = "kebab-case")]
enum Animal {
    Dog { age: u32 },
    Fish,
}

#[test]
fn derives_through_renamed_musli_core() -> Result<(), musli::json::Error> {
    let person = Person {
        name: "Aristotle",
        age: 61,
    };

    let json = musli::json::to_string(&person)?;
    assert_eq!(json, r#"{"name":"Aristotle","age":61}"#);
    assert_eq!(musli::json::from_str::<Person<'_>>(&json)?, person);

    for animal in [Animal::Dog { age: 3 }, Animal::Fish] {
        let json = musli::json::to_string(&animal)?;
        assert_eq!(musli::json::from_str::<Animal>(&json)?, animal);
    }

    assert_eq!(
        musli::json::to_string(&Animal::Dog { age: 3 })?,
        r#"{"dog":{"age":3}}"#
    );
    Ok(())
}
