# musli-serde

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-serde.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-serde)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--serde-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-serde)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Transparent shim to use [`serde`] types in Müsli.

This conveniently and transparently allows Müsli to use fields which are
serde types by marking them with  `#[musli(with = musli_serde)]`. This can
be useful because there is a wide ecosystem of types which implements serde
traits.

Note that the exact method that fields are serialized and deserialized will
not match what Müsli does, since serde requires the use of a fundamentally
different model and Müsli metadata such as `#[musli(name = ..)]` is not
available in [`serde`].

<br>

## Examples

```rust
use serde::{Serialize, Deserialize};
use musli::{Encode, Decode};
use url::Url;

#[derive(Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Encode, Decode)]
#[musli(default_field = "name")]
struct Person {
    name: String,
    #[musli(with = musli_serde)]
    address: Address,
    #[musli(with = musli_serde)]
    url: Url,
}
```

A compatible Müsli structure would look like this:

```rust
use musli::{Encode, Decode};
use url::Url;

#[derive(Encode, Decode)]
#[musli(default_field = "name")]
struct MusliAddress {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Encode, Decode)]
#[musli(default_field = "name")]
struct MusliPerson {
    name: String,
    address: MusliAddress,
    url: String,
}

let json = musli_json::to_string(&Person {
    name: "John Doe".to_string(),
    address: Address {
        street: "Main St.".to_string(),
        city: "Springfield".to_string(),
        zip: 12345,
    },
    url: Url::parse("https://example.com")?,
})?;

let musli = musli_json::from_str::<MusliPerson>(&json)?;

assert_eq!(musli.name, "John Doe");
assert_eq!(musli.address.street, "Main St.");
assert_eq!(musli.address.city, "Springfield");
assert_eq!(musli.address.zip, 12345);
assert_eq!(musli.url, "https://example.com/");
```

[`serde`]: https://serde.rs
