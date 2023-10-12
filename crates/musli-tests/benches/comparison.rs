use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use musli_tests::models::*;
use musli_tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = musli_tests::rng();

    macro_rules! group {
        ($group_name:expr, $name:ident, $it:ident) => {{
            let mut g = c.benchmark_group($group_name);

            macro_rules! bench {
                ($framework:ident, $buf:ident) => {{
                    musli_tests::if_supported! {
                        $framework, $name, {
                            let mut $buf = utils::$framework::new();
                            g.bench_function(stringify!($framework), |b| $it!(b, $framework, $buf));
                        }
                    }
                }};
            }

            musli_tests::feature_matrix!(bench);
        }};
    }

    macro_rules! setup {
        ($($name:ident, $ty:ty, $num:expr, $size_hint:expr),*) => {
            $({
                let $name: $ty = Generate::generate(&mut rng);

                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            state.reset($size_hint, &$name);

                            $b.iter(|| {
                                black_box(state.encode(&$name).unwrap());
                            })
                        })
                    }};
                }

                group!(concat!("enc/", stringify!($name)), $name, it);

                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            state.reset($size_hint, &$name);
                            let data = state.encode(&$name).unwrap();
                            $b.iter(move || data.decode::<$ty>().unwrap())
                        })
                    }};
                }

                group!(concat!("dec/", stringify!($name)), $name, it);

                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            state.reset($size_hint, &$name);

                            $b.iter(|| {
                                let out = state.encode(&$name).unwrap();
                                let actual = out.decode::<$ty>().unwrap();
                                debug_assert_eq!(actual, $name);
                                black_box(actual);
                            })
                        })
                    }};
                }

                group!(concat!("rt/", stringify!($name)), $name, it);
            })*
        };
    }

    musli_tests::types!(setup);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
