use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use musli_tests::models::{Generate, LargeStruct, Primitives};
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let primitives_struct: Primitives = rng.generate();
    let large_struct: LargeStruct = rng.generate();

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
            $b.iter(|| utils::$base::encode(&primitives_struct))
        };
    }

    group!("enc-prim", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {{
            let data = utils::$base::encode(&primitives_struct);
            $b.iter(|| utils::$base::decode::<Primitives>(&data))
        }};
    }

    group!("dec-prim", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| {
                let out = utils::$base::encode(&primitives_struct);
                let actual = utils::$base::decode::<Primitives>(&out);
                debug_assert_eq!(actual, primitives_struct);
                criterion::black_box(actual);
                out
            })
        };
    }

    group!("rt-prim", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {
            $b.iter(|| utils::$base::encode(&large_struct))
        };
    }

    group!("enc-lg", it);

    macro_rules! it {
        ($b:expr, $base:ident) => {{
            let data = utils::$base::encode(&large_struct);
            $b.iter(|| utils::$base::decode::<LargeStruct>(&data))
        }};
    }

    group!("dec-lg", it);

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

    group!("rt-lg", it);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
