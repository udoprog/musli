use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use musli_tests::models::*;
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = musli_tests::rng();

    macro_rules! group {
        ($name:expr, $it:ident) => {{
            let mut g = c.benchmark_group($name);

            macro_rules! bench {
                ($base:ident, $buf:ident) => {{
                    let mut $buf = utils::$base::buffer();
                    g.bench_function(stringify!($base), |b| $it!(b, $base, $buf));
                }};
            }

            musli_tests::feature_matrix!(bench);
        }};
    }

    macro_rules! setup {
        ($($var:ident, $ty:ty, $num:expr, $size_hint:expr),*) => {
            $({
                let $var: $ty = Generate::generate(&mut rng);

                macro_rules! it {
                    ($b:expr, $base:ident, $buf:ident) => {{
                        utils::$base::reset(&mut $buf, $size_hint, &$var);
                        $b.iter(|| {
                            black_box(utils::$base::encode(&mut $buf, &$var).unwrap());
                        })
                    }};
                }

                group!(concat!("enc/", stringify!($var)), it);

                macro_rules! it {
                    ($b:expr, $base:ident, $buf:ident) => {{
                        utils::$base::reset(&mut $buf, $size_hint, &$var);
                        let data = utils::$base::encode(&mut $buf, &$var).unwrap();
                        $b.iter(|| utils::$base::decode::<$ty>(&data).unwrap())
                    }};
                }

                group!(concat!("dec/", stringify!($var)), it);

                macro_rules! it {
                    ($b:expr, $base:ident, $buf:ident) => {{
                        utils::$base::reset(&mut $buf, $size_hint, &$var);

                        $b.iter(|| {
                            let out = utils::$base::encode(&mut $buf, &$var).unwrap();
                            let actual = utils::$base::decode::<$ty>(&out).unwrap();
                            debug_assert_eq!(actual, $var);
                            black_box(actual);
                            black_box(out);
                        })
                    }};
                }

                group!(concat!("rt/", stringify!($var)), it);
            })*
        };
    }

    musli_tests::types!(setup);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
