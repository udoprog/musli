use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use musli_tests::models::{Allocated, Generate, LargeStruct, Primitives};
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let prim: Primitives = rng.generate();
    let alloc: Allocated = rng.generate();
    let lg: LargeStruct = rng.generate();

    macro_rules! group {
        ($name:expr, $it:ident) => {{
            let mut g = c.benchmark_group($name);

            macro_rules! bench {
                ($base:ident) => {{
                    g.bench_function(stringify!($base), |b| $it!(b, $base));
                }};
            }

            musli_tests::feature_matrix!(bench);
        }};
    }

    macro_rules! setup {
        ($var:ident, $ty:ty) => {{
            macro_rules! it {
                ($b:expr, $base:ident) => {
                    $b.iter(|| utils::$base::encode(&$var))
                };
            }

            group!(concat!("enc-", stringify!($var)), it);

            macro_rules! it {
                ($b:expr, $base:ident) => {{
                    let data = utils::$base::encode(&$var);
                    $b.iter(|| utils::$base::decode::<$ty>(&data))
                }};
            }

            group!(concat!("dec-", stringify!($var)), it);

            macro_rules! it {
                ($b:expr, $base:ident) => {
                    $b.iter(|| {
                        let out = utils::$base::encode(&$var);
                        let actual = utils::$base::decode::<$ty>(&out);
                        debug_assert_eq!(actual, $var);
                        criterion::black_box(actual);
                        out
                    })
                };
            }

            group!(concat!("rt-", stringify!($var)), it);
        }};
    }

    setup!(prim, Primitives);
    setup!(alloc, Allocated);
    setup!(lg, LargeStruct);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
