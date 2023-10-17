#[allow(unused)]
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use tests::generate::Generate;
use tests::models::*;
#[allow(unused)]
use tests::utils;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = tests::rng();

    macro_rules! group {
        ($group_name:expr, $name:ident, $it:ident) => {{
            #[allow(unused)]
            let mut g = c.benchmark_group($group_name);

            #[allow(unused)]
            macro_rules! bench {
                ($framework:ident, $buf:ident) => {{
                    tests::if_supported! {
                        $framework, $name, {
                            let mut $buf = utils::$framework::new();
                            g.bench_function(stringify!($framework), |b| $it!(b, $framework, $buf));
                        }
                    }
                }};
            }

            tests::feature_matrix!(bench);
        }};
    }

    macro_rules! setup {
        ($($name:ident, $ty:ty, $num:expr, $size_hint:expr),*) => {
            $({
                let $name: $ty = Generate::generate(&mut rng);

                #[allow(unused)]
                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            $b.iter(|| {
                                state.reset($size_hint, &$name);
                                let _ = black_box(state.encode(&$name).unwrap());
                            });
                        });
                    }};
                }

                group!(concat!("enc/", stringify!($name)), $name, it);

                #[allow(unused)]
                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            state.reset($size_hint, &$name);
                            let data = state.encode(&$name).unwrap();
                            $b.iter(move || data.decode::<$ty>().unwrap());
                        });
                    }};
                }

                group!(concat!("dec/", stringify!($name)), $name, it);

                #[allow(unused)]
                macro_rules! it {
                    ($b:expr, $framework:ident, $buf:ident) => {{
                        $buf.with(|mut state| {
                            $b.iter(|| {
                                state.reset($size_hint, &$name);
                                let out = state.encode(&$name).unwrap();
                                let actual = out.decode::<$ty>().unwrap();
                                debug_assert_eq!(actual, $name);
                                black_box(actual);
                            });
                        })
                    }};
                }

                #[cfg(not(feature = "no-rt"))]
                group!(concat!("rt/", stringify!($name)), $name, it);
            })*
        };
    }

    tests::types!(setup);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
