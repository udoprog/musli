use std::collections::HashMap;
use std::fmt::Debug;

use criterion::{criterion_group, criterion_main, Criterion};
use musli::{Decode, Encode};
use musli_wire::{Fixed, FixedLength, WireEncoding};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(packed)]
struct SmallStruct {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[musli(transparent)]
    Variant2(f64),
    #[musli(transparent)]
    Variant3(u64),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(packed)]
struct BigStruct {
    elements: Vec<SmallStruct>,
    values: Vec<MediumEnum>,
    map: HashMap<u32, u32>,
}

fn generate_big_struct() -> BigStruct {
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(123412327832);

    let mut elements = Vec::new();
    let mut values = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(SmallStruct {
            x: rng.gen(),
            y: rng.gen(),
        });
    }

    for _ in 0..rng.gen_range(100usize..500) {
        values.push(match rng.gen_range(0..=2) {
            0 => MediumEnum::Variant1(format!("Hello {}", rng.gen_range(100000..500000))),
            1 => MediumEnum::Variant2(rng.gen()),
            _ => MediumEnum::Variant3(rng.gen()),
        });
    }

    let mut map = HashMap::new();

    for _ in 0..342 {
        map.insert(rng.gen(), rng.gen());
    }

    BigStruct {
        elements,
        values,
        map,
    }
}

#[allow(unused)]
fn rmp_rt<T>(expected: &T) -> T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let data = rmp_serde::to_vec(expected).unwrap();
    let value: T = rmp_serde::from_slice(&data[..]).unwrap();
    value
}

#[allow(unused)]
fn rmp_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    rmp_serde::to_vec(expected).unwrap()
}

#[allow(unused)]
fn rmp_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    rmp_serde::from_slice(data).unwrap()
}

fn bin_rt<T>(expected: &T) -> T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let data = bincode::serialize(expected).unwrap();
    let value: T = bincode::deserialize(&data[..]).unwrap();
    value
}

fn bin_enc<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    bincode::serialize(expected).unwrap()
}

fn bin_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    bincode::deserialize(data).unwrap()
}

fn json_enc<T>(value: &T) -> Vec<u8>
where
    T: Serialize,
{
    serde_json::to_vec(value).unwrap()
}

fn json_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    serde_json::from_slice(data).unwrap()
}

fn json_rt<T>(expected: &T) -> T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let data = serde_json::to_vec(expected).unwrap();
    let value: T = serde_json::from_slice(&data[..]).unwrap();
    value
}

const ENCODING: WireEncoding<Fixed, FixedLength> = WireEncoding::new()
    .with_fixed_integers()
    .with_fixed_lengths();

fn musli_rt<T>(expected: &T) -> T
where
    T: Encode + for<'de> Decode<'de>,
{
    let data = ENCODING.to_vec(expected).unwrap();
    let value: T = ENCODING.decode(&data[..]).unwrap();
    value
}

fn musli_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    ENCODING.to_vec(expected).unwrap()
}

fn musli_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    ENCODING.decode(data).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let small_struct = SmallStruct { x: 32.0, y: 64.0 };
    let big_struct = generate_big_struct();

    macro_rules! benches {
        ($group:literal, $encode_fn:ident, $decode_fn:ident, $roundtrip_fn:ident) => {{
            let mut group = c.benchmark_group($group);

            let small_data = $encode_fn(&big_struct);

            group.bench_function("roundtrip-small", |b| {
                b.iter(|| $roundtrip_fn(&small_struct))
            });
            group.bench_function("encode-small", |b| b.iter(|| $encode_fn(&small_struct)));
            group.bench_function("decode-small", |b| {
                b.iter(|| $decode_fn::<SmallStruct>(&small_data))
            });

            let big_data = $encode_fn(&big_struct);

            group.bench_function("roundtrip-big", |b| b.iter(|| $roundtrip_fn(&big_struct)));
            group.bench_function("encode-big", |b| b.iter(|| $encode_fn(&big_struct)));
            group.bench_function("decode-big", |b| {
                b.iter(|| $decode_fn::<BigStruct>(&big_data))
            });
        }};
    }

    // benches!("rmp-serde", rmp_enc, rmp_dec, rmp_rt);
    benches!("bincode-serde", bin_enc, bin_dec, bin_rt);
    benches!("musli", musli_enc, musli_dec, musli_rt);
    benches!("json", json_enc, json_dec, json_rt);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
