/// Generate asm with:
///
/// ```sh
/// cargo rustc --manifest-path=crates/musli-tests/Cargo.toml --release --example storage-optimize -- --emit asm
/// ```
use std::hint::black_box;

use musli::{mode::DefaultMode, Decode, Encode};
use musli_storage::encoding::Encoding;
use musli_storage::int::{Fixed, NativeEndian, Variable};

use anyhow::{Context, Result};

const ENCODING: Encoding<DefaultMode, Fixed<NativeEndian>, Variable> =
    Encoding::new().with_fixed_integers_endian();

#[derive(Encode, Decode)]
#[musli(packed)]
pub struct Storage {
    value: u32,
    value2: u32,
}

#[inline(never)]
#[no_mangle]
pub fn with_musli(storage: &Storage) -> Result<[u8; 8]> {
    Ok(ENCODING
        .to_fixed_bytes::<8, _>(storage)?
        .into_bytes()
        .context("Buffer not filled")?)
}

#[inline(never)]
#[no_mangle]
pub fn without_musli(storage: &Storage) -> Result<[u8; 8]> {
    let [a, b, c, d] = storage.value.to_ne_bytes();
    let [e, f, g, h] = storage.value2.to_ne_bytes();
    Ok([a, b, c, d, e, f, g, h])
}

fn main() -> Result<()> {
    let storage = Storage {
        value: 4,
        value2: 8,
    };

    black_box(with_musli(&storage)?);
    black_box(without_musli(&storage)?);
    Ok(())
}
