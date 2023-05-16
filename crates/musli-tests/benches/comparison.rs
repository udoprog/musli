use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use musli_tests::models::{LargeStruct, Primitives};
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let primitives_struct = musli_tests::models::generate_primitives(&mut rng);
    let large_struct = musli_tests::models::generate_large_struct(&mut rng);

    macro_rules! group {
        ($name:literal, $it:ident) => {{
            let mut g = c.benchmark_group($name);

            macro_rules! bench {
                ($base:ident) => {{
                    g.bench_function(stringify!($base), |b| $it!(b, $base));
                }};
            }

            musli_tests::feature_matrix!(bench);
        }};
    }

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| {
                let out = utils::$base::encode(&primitives_struct);
                let actual = utils::$base::decode::<Primitives>(&out);
                debug_assert_ne!(actual, primitives_struct);
                criterion::black_box(actual);
                out
            })
        };
    }

    group!("roundtrip-primitives", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| {
                let out = utils::$base::encode(&large_struct);
                let actual = utils::$base::decode::<LargeStruct>(&out);
                debug_assert_eq!(actual, large_struct);
                criterion::black_box(actual);
                out
            })
        };
    }

    group!("roundtrip-large", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| utils::$base::encode(&primitives_struct))
        };
    }

    group!("encode-primitives", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| utils::$base::encode(&primitives_struct))
        };
    }

    group!("encode-primitives", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| utils::$base::encode(&large_struct))
        };
    }

    group!("encode-large", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {{
            let data = utils::$base::encode(&primitives_struct);
            $b.iter(|| utils::$base::decode::<Primitives>(&data))
        }};
    }

    group!("decode-primitives", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {{
            let data = utils::$base::encode(&large_struct);
            $b.iter(|| utils::$base::decode::<LargeStruct>(&data))
        }};
    }

    group!("decode-large", it);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
