use std::hint::black_box;

use criterion::Criterion;

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

                        let mut out = match state.encode(value) {
                            Ok(out) => out,
                            Err(error) => {
                                panic! {
                                    "{} / {}: encoding of value[{index}] failed: {error:?}",
                                    stringify!($framework),
                                    stringify!($name)
                                };
                            }
                        };

                        let actual = match out.decode::<$ty>() {
                            Ok(actual) => actual,
                            Err(error) => {
                                panic! {
                                    "{} / {}: decoding of value[{index}] failed: {error:?}",
                                    stringify!($framework),
                                    stringify!($name)
                                };
                            }
                        };

                        #[cfg(not(feature = "no-binary-equality"))]
                        assert_eq!(
                            actual,
                            *value,
                            "{} / {}: roundtrip encoding of value[{index}] should be equal",
                            stringify!($framework),
                            stringify!($name)
                        );

                        #[cfg(feature = "no-binary-equality")]
                        {
                            _ = (actual, index);
                        }
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

            #[cfg(not(feature = "no-rt"))]
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

fn main() {
    // SAFETY: This is only called once at the beginning of the program.
    unsafe {
        tests::init_statics();
    }

    let mut c = Criterion::default().configure_from_args();

    criterion_benchmark(&mut c);

    if std::env::var_os("MUSLI_FINAL_SUMMARY").is_none_or(|value| value != "no") {
        c.final_summary();
    }
}
