#![cfg(not(miri))]

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Instant;

use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_json::JsonEncoding;
use musli_wire::WireEncoding;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

const SMALL_ITERATIONS: usize = 100_000;
const LARGE_ITERATIONS: usize = 1_000;

#[test]
fn test_fuzz() {
    let mut rng = StdRng::seed_from_u64(123412327832);

    macro_rules! test {
        ($group:literal, $encode_fn:ident, $decode_fn:ident) => {{
            let start = Instant::now();

            for _ in 0..SMALL_ITERATIONS {
                let small_struct = generate_small_struct(&mut rng);

                let out = $encode_fn(&small_struct);
                let actual = $decode_fn::<SmallStruct>(&out[..]);
                assert_eq!(actual, small_struct);
            }

            for _ in 0..LARGE_ITERATIONS {
                let large_struct = generate_large_struct(&mut rng);

                let out = $encode_fn(&large_struct);
                let actual = $decode_fn::<LargeStruct>(&out[..]);
                assert_eq!(actual, large_struct);
            }

            println!("{}: {:?}", $group, Instant::now().duration_since(start));
        }};
    }

    test!("musli-json", musli_json_enc, musli_json_dec);
    test!("musli-storage", musli_storage_enc, musli_storage_dec);
    test!("musli-wire", musli_wire_enc, musli_wire_dec);
}

const JSON_ENCODING: JsonEncoding = JsonEncoding::new();

fn musli_json_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    JSON_ENCODING.to_vec(expected).unwrap()
}

fn musli_json_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    JSON_ENCODING.from_slice(data).unwrap()
}

const WIRE_ENCODING: WireEncoding<DefaultMode, musli_wire::Fixed, musli_wire::FixedLength> =
    WireEncoding::new()
        .with_fixed_integers()
        .with_fixed_lengths();

fn musli_wire_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    // NB: bincode uses a 128-byte pre-allocated vector.
    let mut data = Vec::with_capacity(128);
    WIRE_ENCODING.encode(&mut data, expected).unwrap();
    data
}

fn musli_wire_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    WIRE_ENCODING.decode(data).unwrap()
}

const STORAGE_ENCODING: musli_storage::StorageEncoding<
    DefaultMode,
    musli_storage::Fixed,
    musli_storage::FixedLength,
> = musli_storage::StorageEncoding::new()
    .with_fixed_integers()
    .with_fixed_lengths();

fn musli_storage_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    // NB: bincode uses a 128-byte pre-allocated vector.
    let mut data = Vec::with_capacity(128);
    STORAGE_ENCODING.encode(&mut data, expected).unwrap();
    data
}

fn musli_storage_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de, DefaultMode>,
{
    STORAGE_ENCODING.from_slice(data).unwrap()
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
struct SmallStruct {
    a: u32,
    b: u64,
    c: u128,
    d: f32,
    e: f64,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[musli(transparent)]
    Variant2(u128),
    #[musli(transparent)]
    Variant3(u64),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
struct LargeStruct {
    elements: Vec<SmallStruct>,
    medium: Vec<MediumEnum>,
    map: HashMap<u32, u64>,
}

fn generate_small_struct(rng: &mut StdRng) -> SmallStruct {
    SmallStruct {
        a: rng.gen(),
        b: rng.gen(),
        c: rng.gen(),
        d: rng.gen(),
        e: rng.gen(),
    }
}

fn generate_string(rng: &mut StdRng) -> String {
    format!("Hello {}", rng.gen_range(100000..500000))
}

fn generate_medium_enum(rng: &mut StdRng) -> MediumEnum {
    match rng.gen_range(0..=2) {
        0 => MediumEnum::Variant1(generate_string(rng)),
        1 => MediumEnum::Variant2(rng.gen()),
        _ => MediumEnum::Variant3(rng.gen()),
    }
}

fn generate_large_struct(rng: &mut StdRng) -> LargeStruct {
    let mut elements = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(generate_small_struct(rng));
    }

    let mut medium = Vec::new();

    for _ in 0..rng.gen_range(200..500) {
        medium.push(generate_medium_enum(rng));
    }

    let mut map = HashMap::new();

    for _ in 0..342 {
        map.insert(rng.gen(), rng.gen());
    }

    LargeStruct {
        elements,
        medium,
        map,
    }
}
