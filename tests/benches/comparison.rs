use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use tests::models::*;
use tests::utils;
use tests::Generate;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = tests::rng();

    macro_rules! for_each {
        ($name:ident, $call:ident) => {{
            macro_rules! inner {
                ($framework:ident) => {{
                    tests::if_supported! {
                        $framework, $name, {
                            if utils::$framework::is_enabled() {
                                $call!($framework)
                            }
                        }
                    }
                }};
            }

            tests::feature_matrix!(inner);
        }};
    }

    macro_rules! group {
        ($bench:ident, $name:ident, $it:ident) => {{
            let mut g = c.benchmark_group(concat!(stringify!($bench), "/", stringify!($name)));

            macro_rules! inner {
                ($framework:ident) => {{
                    tests::if_supported! {
                        $framework, $name, {
                            if utils::$framework::is_enabled() {
                                g.bench_function(stringify!($framework), |b| $it!(b, $framework));
                            }
                        }
                    }
                }};
            }

            tests::feature_matrix!(inner);
        }};
    }

    macro_rules! setup {
        ($name:ident, $ty:ty, $num:expr, $size_hint:expr) => {{
            let mut values = Vec::<$ty>::new();
            Generate::generate_in(&mut rng, |value| values.push(value));

            macro_rules! check {
                ($framework:ident) => {{
                    let mut frameworks = Vec::with_capacity(values.len());

                    for _ in &values {
                        frameworks.push(utils::$framework::new());
                    }

                    for (index, (value, framework)) in
                        values.iter().zip(&mut frameworks).enumerate()
                    {
                        let mut state = framework.state();
                        state.reset($size_hint, value);
                        let mut out = state.encode(value).expect("encoding should success");

                        let actual = out.decode::<$ty>().expect("decoding should succeed");

                        #[cfg(not(feature = "no-binary-equality"))]
                        assert_eq!(
                            actual,
                            *value,
                            "{} / {}: roundtrip encoding of value[{index}] should be equal",
                            stringify!($framework),
                            stringify!($name)
                        );

                        #[cfg(feature = "no-binary-equality")]
                        drop((actual, index));
                    }
                }};
            }

            for_each!($name, check);

            macro_rules! it {
                ($b:expr, $framework:ident) => {{
                    let mut frameworks = Vec::with_capacity(values.len());

                    for _ in &values {
                        frameworks.push(utils::$framework::new());
                    }

                    $b.iter(|| {
                        for (value, framework) in values.iter().zip(&mut frameworks) {
                            let mut state = framework.state();
                            state.reset($size_hint, value);
                            black_box(state.encode(value).unwrap());
                        }
                    });
                }};
            }

            group!(enc, $name, it);

            macro_rules! it {
                ($b:expr, $framework:ident) => {{
                    let mut frameworks = Vec::with_capacity(values.len());

                    for _ in &values {
                        frameworks.push(utils::$framework::new());
                    }

                    let mut states = Vec::with_capacity(values.len());

                    for framework in &mut frameworks {
                        states.push(framework.state());
                    }

                    let mut inputs = Vec::with_capacity(values.len());

                    for (value, state) in values.iter().zip(&mut states) {
                        state.reset($size_hint, value);
                        inputs.push(state.encode(value).unwrap());
                    }

                    $b.iter(move || {
                        for data in &mut inputs {
                            black_box(data.decode::<$ty>().unwrap());
                        }
                    });
                }};
            }

            group!(dec, $name, it);

            macro_rules! it {
                ($b:expr, $framework:ident) => {{
                    if utils::$framework::is_enabled() {
                        let mut frameworks = Vec::with_capacity(values.len());

                        for _ in &values {
                            frameworks.push(utils::$framework::new());
                        }

                        $b.iter(|| {
                            for (value, framework) in values.iter().zip(&mut frameworks) {
                                let mut state = framework.state();
                                state.reset($size_hint, value);
                                let mut out = black_box(state.encode(value).unwrap());
                                let actual = black_box(out.decode::<$ty>().unwrap());
                                debug_assert_eq!(actual, *value);
                                black_box(actual);
                            }
                        });
                    }
                }};
            }

            #[cfg(not(feature = "no-rt"))]
            group!(rt, $name, it);
        }};
    }

    tests::types!(setup);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
