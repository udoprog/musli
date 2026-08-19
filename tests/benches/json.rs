//! Targeted benchmarks for JSON parsing primitives.
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p tests --bench json --features musli-json,serde_json
//! ```

#[cfg(all(feature = "musli-json", feature = "serde_json"))]
use std::hint::black_box;

use criterion::Criterion;

#[cfg(all(feature = "musli-json", feature = "serde_json"))]
macro_rules! group {
    ($c:expr, $name:expr, $ty:ty, $data:expr) => {{
        let data: &str = $data;
        let mut g = $c.benchmark_group($name);
        g.throughput(criterion::Throughput::Bytes(data.len() as u64));

        // Sanity check that both frameworks agree on the payload.
        let a = musli::json::from_str::<$ty>(data).unwrap();
        let b = serde_json::from_str::<$ty>(data).unwrap();
        assert_eq!(a, b);

        g.bench_function("musli", |b| {
            b.iter(|| musli::json::from_str::<$ty>(black_box(data)).unwrap())
        });

        g.bench_function("serde_json", |b| {
            b.iter(|| serde_json::from_str::<$ty>(black_box(data)).unwrap())
        });

        g.finish();
    }};
}

#[cfg(all(feature = "musli-json", feature = "serde_json"))]
#[derive(Debug, PartialEq, musli::Encode, musli::Decode, serde::Serialize, serde::Deserialize)]
struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[cfg(all(feature = "musli-json", feature = "serde_json"))]
fn array<T>(values: impl IntoIterator<Item = T>) -> String
where
    T: std::fmt::Display,
{
    let mut out = String::from("[");

    for (n, value) in values.into_iter().enumerate() {
        if n > 0 {
            out.push(',');
        }

        out.push_str(&value.to_string());
    }

    out.push(']');
    out
}

#[cfg(all(feature = "musli-json", feature = "serde_json"))]
fn criterion_benchmark(c: &mut Criterion) {
    use rand::{RngExt, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(tests::RNG_SEED);

    // Single digit unsigned integers.
    let tiny = array((0..4096).map(|_| rng.random_range(0u8..10)));
    group!(c, "u8-tiny", Vec<u8>, &tiny);

    // Small unsigned integers, 1-5 digits.
    let small = array((0..4096).map(|_| rng.random_range(0u32..100_000)));
    group!(c, "u32-small", Vec<u32>, &small);

    // Large unsigned integers, mostly 19-20 digits.
    let large = array((0..4096).map(|_| rng.random_range(u64::MAX / 2..u64::MAX)));
    group!(c, "u64-large", Vec<u64>, &large);

    // Signed integers with mixed signs.
    let signed = array((0..4096).map(|_| rng.random_range(i64::MIN / 2..i64::MAX / 2)));
    group!(c, "i64-mixed", Vec<i64>, &signed);

    // Floats.
    let floats = array((0..4096).map(|_| rng.random_range(-1.0e6f64..1.0e6f64)));
    group!(c, "f64", Vec<f64>, &floats);

    let floats = array((0..4096).map(|_| rng.random_range(-1.0e6f32..1.0e6f32)));
    group!(c, "f32", Vec<f32>, &floats);

    // Small objects with float fields, mirroring the mesh model.
    let objects = array((0..1024).map(|_| {
        let mut f = || rng.random_range(-1.0e6f32..1.0e6f32);
        format!(r#"{{"x":{},"y":{},"z":{}}}"#, f(), f(), f())
    }));
    group!(c, "objects", Vec<Vector>, &objects);

    // Plain strings which require no unescaping.
    let strings = array((0..4096).map(|_| {
        let len = rng.random_range(1usize..24);
        let s: String = (0..len).map(|_| rng.random_range('a'..='z')).collect();
        format!("\"{s}\"")
    }));
    group!(c, "strings", Vec<String>, &strings);

    // Strings which contain escape sequences.
    let escaped = array((0..4096).map(|_| {
        let len = rng.random_range(1usize..24);
        let s: String = (0..len)
            .map(|_| {
                if rng.random_range(0u8..4) == 0 {
                    "\\n".to_string()
                } else {
                    rng.random_range('a'..='z').to_string()
                }
            })
            .collect();
        format!("\"{s}\"")
    }));
    group!(c, "strings-escaped", Vec<String>, &escaped);
}

#[cfg(not(all(feature = "musli-json", feature = "serde_json")))]
fn criterion_benchmark(_: &mut Criterion) {
    eprintln!("Skipping: requires --features musli-json,serde_json");
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
