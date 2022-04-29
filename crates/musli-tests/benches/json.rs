use std::collections::HashMap;
use std::fmt::Debug;

use criterion::{criterion_group, criterion_main, Criterion};
use musli::{mode::DefaultMode, Decode, Encode};
use musli_json::JsonEncoding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
struct SmallStruct {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
struct LargeStruct {
    elements: Vec<SmallStruct>,
    map: HashMap<u32, u32>,
}

fn generate_large_struct() -> LargeStruct {
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(123412327832);

    let mut elements = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(SmallStruct {
            x: rng.gen(),
            y: rng.gen(),
        });
    }

    let mut map = HashMap::new();

    for _ in 0..342 {
        map.insert(rng.gen(), rng.gen());
    }

    LargeStruct { elements, map }
}

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

const ENCODING: JsonEncoding = JsonEncoding::new();

fn musli_json_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode<DefaultMode>,
{
    ENCODING.to_vec(expected).unwrap()
}

fn musli_json_dec<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    ENCODING.from_slice(data).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let small_struct = SmallStruct { x: 32, y: 64 };
    let large_struct = generate_large_struct();

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
        }};
    }

    benches!("serde-json", serde_json_enc, serde_json_dec);
    benches!("musli-json", musli_json_enc, musli_json_dec);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
