use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use musli_tests::models::Primitives;
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let primitives_struct = musli_tests::models::generate_primitives(&mut rng);
    let large_struct = musli_tests::models::generate_large_struct(&mut rng);

    macro_rules! benches {
        ($base:ident) => {{
            let mut group = c.benchmark_group(stringify!($base));
            group.bench_function("encode-primitives", |b| {
                b.iter(|| utils::$base::encode(&primitives_struct))
            });
            group.bench_function("encode-large", |b| {
                b.iter(|| utils::$base::encode(&large_struct))
            });

            group.bench_function("decode-primitives", |b| {
                let primitives_data = utils::$base::encode(&primitives_struct);
                b.iter(|| utils::$base::decode::<Primitives>(&primitives_data))
            });
            group.bench_function("decode-large", |b| {
                let large_data = utils::$base::encode(&large_struct);
                b.iter(|| utils::$base::decode::<musli_tests::models::LargeStruct>(&large_data))
            });

            group.bench_function("roundtrip-primitives", |b| {
                b.iter(|| {
                    let out = utils::$base::encode(&primitives_struct);
                    let actual = utils::$base::decode::<Primitives>(&out);
                    // assert_eq!(actual, primitives_struct);
                    actual
                })
            });
            group.bench_function("roundtrip-large", |b| {
                b.iter(|| {
                    let out = utils::$base::encode(&large_struct);
                    let actual = utils::$base::decode::<musli_tests::models::LargeStruct>(&out);
                    // assert_eq!(actual, large_struct);
                    actual
                })
            });
        }};
    }

    benches!(serde_json);
    benches!(serde_bincode);
    benches!(serde_rmp);
    benches!(musli_json);
    benches!(musli_wire);
    benches!(musli_descriptive);
    benches!(musli_storage);
    benches!(musli_value);
    #[cfg(not(any(model_128, model_all)))]
    benches!(serde_dlhn);
    benches!(serde_cbor);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
