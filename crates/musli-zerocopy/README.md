# musli-zerocopy

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-zerocopy.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-zerocopy)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--zerocopy-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-zerocopy)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

Zero copy primitives for use in Müsli.

This provides a base set of tools to deal with types which do not require
copying during deserialization.

<br>

## Examples

```rust
use musli_zerocopy::{OwnedBuf, Pair, Unsized, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Unsized<str>,
}

let mut buf = OwnedBuf::new();

let string = buf.insert_unsized("string")?;

let custom1 = buf.insert_sized(Custom { field: 1, string })?;
let custom2 = buf.insert_sized(Custom { field: 2, string })?;

let mut map = Vec::new();

map.push(Pair::new(1, custom1));
map.push(Pair::new(2, custom2));

let map = buf.insert_map(&mut map)?;

let buf = buf.as_aligned_buf();

let custom1 = map.get(buf, &1)?.expect("Missing key 1");
let custom1 = buf.load(custom1)?;
assert_eq!(custom1.field, 1);
assert_eq!(buf.load(custom1.string)?, "string");

let custom2 = map.get(buf, &2)?.expect("Missing key 2");
let custom2 = buf.load(custom2)?;
assert_eq!(custom2.field, 2);
assert_eq!(buf.load(custom2.string)?, "string");

assert!(map.get(buf, &3)?.is_none());
Ok::<_, musli_zerocopy::Error>(())
```
