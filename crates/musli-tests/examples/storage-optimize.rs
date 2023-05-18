/// Generate asm with:
///
/// ```sh
/// cargo rustc --manifest-path=crates/musli-tests/Cargo.toml --release --example storage-optimize -- --emit asm
/// ```
use std::hint::black_box;

use musli::{mode::DefaultMode, Decode, Encode};
use musli_storage::encoding::Encoding;
use musli_storage::int::{Fixed, NativeEndian, Variable};

use anyhow::Result;

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
    let mut array = [0; 8];
    ENCODING.encode(&mut array[..], storage)?;
    Ok(array)
}

#[inline(never)]
#[no_mangle]
pub fn without_musli(storage: &Storage) -> Result<[u8; 8]> {
    let mut array = [0; 8];
    array[..4].copy_from_slice(&storage.value.to_ne_bytes());
    array[4..].copy_from_slice(&storage.value2.to_ne_bytes());
    Ok(array)
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
