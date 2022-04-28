use std::collections::HashMap;
use std::fmt::Debug;

use criterion::{criterion_group, criterion_main, Criterion};
use musli::{mode::DefaultMode, Decode, Encode};
use musli_json::JsonEncoding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
struct SmallStruct {
    x: f32,
    y: f32,
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

const ENCODING: JsonEncoding = JsonEncoding::new();

fn musli_json_enc<T>(expected: &T) -> Vec<u8>
where
    T: Encode<DefaultMode>,
{
    ENCODING.to_vec(expected).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let small_struct = SmallStruct { x: 32.0, y: 64.0 };
    let large_struct = generate_large_struct();

    macro_rules! benches {
        ($group:literal, $encode_fn:ident) => {{
            let mut group = c.benchmark_group($group);
            group.bench_function("encode-small", |b| b.iter(|| $encode_fn(&small_struct)));
            group.bench_function("encode-large", |b| b.iter(|| $encode_fn(&large_struct)));
        }};
    }

    benches!("serde-json", serde_json_enc);
    benches!("musli-json", musli_json_enc);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
