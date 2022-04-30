use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let small_struct = musli_tests::models::generate_small_struct(&mut rng);
    let large_struct = musli_tests::models::generate_large_struct(&mut rng);

    macro_rules! benches {
        ($base:ident) => {{
            let mut group = c.benchmark_group(stringify!($base));
            group.bench_function("encode-small", |b| {
                b.iter(|| musli_tests::utils::$base::encode(&small_struct))
            });
            group.bench_function("encode-large", |b| {
                b.iter(|| musli_tests::utils::$base::encode(&large_struct))
            });

            let small_data = musli_tests::utils::$base::encode(&small_struct);
            let large_data = musli_tests::utils::$base::encode(&large_struct);

            group.bench_function("decode-small", |b| {
                b.iter(|| {
                    musli_tests::utils::$base::decode::<musli_tests::models::SmallStruct>(
                        &small_data,
                    )
                })
            });
            group.bench_function("decode-large", |b| {
                b.iter(|| {
                    musli_tests::utils::$base::decode::<musli_tests::models::LargeStruct>(
                        &large_data,
                    )
                })
            });

            group.bench_function("roundtrip-small", |b| {
                b.iter(|| {
                    let out = musli_tests::utils::$base::encode(&small_struct);
                    let actual = musli_tests::utils::$base::decode::<
                        musli_tests::models::SmallStruct,
                    >(&out[..]);
                    // assert_eq!(actual, small_struct);
                    actual
                })
            });
            group.bench_function("roundtrip-large", |b| {
                b.iter(|| {
                    let out = musli_tests::utils::$base::encode(&large_struct);
                    let actual = musli_tests::utils::$base::decode::<
                        musli_tests::models::LargeStruct,
                    >(&out[..]);
                    // assert_eq!(actual, large_struct);
                    actual
                })
            });
        }};
    }

    benches!(serde_json);
    benches!(serde_bincode);
    benches!(serde_rmp);
    // benches!(serde_cbor);
    benches!(musli_json);
    benches!(musli_wire);
    benches!(musli_storage);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
