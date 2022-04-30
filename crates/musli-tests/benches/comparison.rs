use std::collections::HashMap;
use std::fmt::Debug;

use criterion::{criterion_group, criterion_main, Criterion};
use musli::{mode::DefaultMode, Decode, Encode};
use musli_json::JsonEncoding;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let small_struct = generate_small_struct(&mut rng);
    let large_struct = generate_large_struct(&mut rng);

    macro_rules! benches {
        ($group:literal, $encode_fn:ident, $decode_fn:ident) => {{
            let mut group = c.benchmark_group($group);
            group.bench_function("encode-small", |b| b.iter(|| $encode_fn(&small_struct)));
            group.bench_function("encode-large", |b| b.iter(|| $encode_fn(&large_struct)));

            let small_data = $encode_fn(&small_struct);
            let large_data = $encode_fn(&large_struct);

            group.bench_function("decode-small", |b| {
                b.iter(|| $decode_fn::<SmallStruct>(&small_data))
            });
            group.bench_function("decode-large", |b| {
                b.iter(|| $decode_fn::<LargeStruct>(&large_data))
            });

            group.bench_function("roundtrip-small", |b| {
                b.iter(|| {
                    let out = $encode_fn(&small_struct);
                    let actual = $decode_fn::<SmallStruct>(&out[..]);
                    // assert_eq!(actual, small_struct);
                    actual
                })
            });
            group.bench_function("roundtrip-large", |b| {
                b.iter(|| {
                    let out = $encode_fn(&large_struct);
                    let actual = $decode_fn::<LargeStruct>(&out[..]);
                    // assert_eq!(actual, large_struct);
                    actual
                })
            });
        }};
    }

    benches!("serde-json", serde_json_enc, serde_json_dec);
    benches!("serde-bincode", serde_bincode_enc, serde_bincode_dec);
    benches!("serde-rmp", serde_rmp_enc, serde_rmp_dec);
    benches!("serde-cbor", serde_cbor_enc, serde_cbor_dec);
    benches!("musli-json", musli_json_enc, musli_json_dec);
    benches!("musli-wire", musli_wire_enc, musli_wire_dec);
    benches!("musli-storage", musli_storage_enc, musli_storage_dec);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn serde_json_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    serde_json::to_vec(expected).unwrap()
}

fn serde_json_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    serde_json::from_slice(data).unwrap()
}

const JSON_ENCODING: JsonEncoding = JsonEncoding::new();

fn musli_json_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode<DefaultMode>,
{
    JSON_ENCODING.to_vec(expected).unwrap()
}

fn musli_json_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    JSON_ENCODING.from_slice(data).unwrap()
}

#[allow(unused)]
fn serde_rmp_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    rmp_serde::to_vec(expected).unwrap()
}

#[allow(unused)]
fn serde_rmp_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    rmp_serde::from_slice(data).unwrap()
}

fn serde_bincode_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    bincode::serialize(expected).unwrap()
}

fn serde_bincode_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    bincode::deserialize(data).unwrap()
}

fn serde_cbor_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    serde_cbor::to_vec(expected).unwrap()
}

fn serde_cbor_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    serde_cbor::from_slice(data).unwrap()
}

const WIRE_ENCODING: musli_wire::WireEncoding<
    DefaultMode,
    musli_wire::Fixed,
    musli_wire::FixedLength,
> = musli_wire::WireEncoding::new()
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
    T: Decode<'de>,
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
