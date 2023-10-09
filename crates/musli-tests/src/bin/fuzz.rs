use std::collections::{BTreeSet, HashMap};
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use musli_tests::models::*;
use musli_tests::utils;

struct SizeSet {
    framework: &'static str,
    suite: &'static str,
    samples: Vec<i64>,
}

musli_tests::miri! {
    const ITER: usize = 10000, 2;
    const LARGE_STRUCTS: usize = 10, 2;
    const PRIMITIVES: usize = 500, 2;
    const MEDIUM_ENUMS: usize = 500, 2;
    const ALLOCATED: usize = 100, 2;
}

fn generate<T>(rng: &mut StdRng, count: usize) -> Vec<T>
where
    T: Generate,
{
    let mut out = Vec::with_capacity(count);

    for _ in 0..count {
        out.push(T::generate(rng));
    }

    out
}

fn main() -> Result<()> {
    let mut rng = musli_tests::rng();

    let mut it = std::env::args().skip(1);

    let mut iter = ITER;
    let mut random = false;
    let mut size = false;
    let mut filter = Vec::new();

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--iter" => {
                iter = it
                    .next()
                    .context("missing argument for `--iter`")?
                    .parse()
                    .context("bad argument to --iter")?;
            }
            "--random" => {
                random = true;
            }
            "--size" => {
                size = true;
            }
            other if other.starts_with("--") => {
                bail!("Bad argument: {other}");
            }
            _ => {
                filter.push(arg);
            }
        }
    }

    let condition = move |name: &str| {
        if filter.is_empty() {
            return true;
        }

        filter.iter().all(|f| name.contains(f))
    };

    let stdout = std::io::stdout();
    let mut o = stdout.lock();

    let mut random_bytes: Vec<Vec<u8>> = Vec::new();
    random_bytes.push(Vec::new());

    if random {
        for _ in 0..iter {
            random_bytes.push(Generate::generate_range(&mut rng, 0..256));
        }
    }

    macro_rules! fuzz {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($base:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                let name = concat!(stringify!($base), "/", stringify!($name), "/random");

                if random && condition(name) {
                    write!(o, "{name}: ")?;
                    o.flush()?;
                    let start = Instant::now();

                    let step = random_bytes.len() / 10;

                    for (n, bytes) in random_bytes.iter().enumerate() {
                        if step == 0 || n % step == 0 {
                            write!(o, ".")?;
                            o.flush()?;
                        }

                        match utils::$base::decode::<$ty>(&bytes) {
                            Ok(value) => {
                                // values *can* occur.
                                black_box(value);
                            }
                            Err(error) => {
                                // errors are expected, so don't log them.
                                black_box(error);
                            }
                        }
                    }

                    let duration = Instant::now().duration_since(start);
                    writeln!(o, " {duration:?}")?;
                }
            })*
        };
    }

    let mut size_sets = Vec::<SizeSet>::new();

    macro_rules! size {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($base:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                let name = concat!(stringify!($base), "/", stringify!($name), "/size");

                if size && condition(name) {
                    let mut buf = utils::$base::buffer();

                    let mut set = SizeSet {
                        framework: stringify!($base),
                        suite: stringify!($name),
                        samples: Vec::new(),
                    };

                    for var in &$name {
                        utils::$base::reset(&mut buf, $size_hint, var);

                        match utils::$base::encode(&mut buf, var) {
                            Ok(value) => {
                                set.samples.push(value.len() as i64);
                            }
                            Err(error) => {
                                writeln!(o, "{name}: error during encode: {error}")?;
                            }
                        }
                    }

                    size_sets.push(set);
                }
            })*
        };
    }

    macro_rules! run {
        ($base:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                let name = concat!(stringify!($base), "/", stringify!($name));

                if (!random && !size) && condition(name) {
                    write!(o, "{name}: ")?;
                    o.flush()?;
                    let start = Instant::now();
                    let step = iter / 10;

                    let mut buf = utils::$base::buffer();

                    'outer:
                    for n in 0..iter {
                        if step == 0 || n % step == 0 {
                            write!(o, ".")?;
                            o.flush()?;
                        }

                        for (index, var) in $name.iter().enumerate() {
                            utils::$base::reset(&mut buf, $size_hint, var);

                            let out = match utils::$base::encode(&mut buf, var) {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during encode: {error}")?;
                                    break 'outer;
                                }
                            };

                            let actual = match utils::$base::decode::<$ty>(&out) {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during decode: {error}")?;
                                    break 'outer;
                                }
                            };

                            if actual != *var {
                                write!(o, "C")?;
                                writeln!(o)?;
                                writeln!(o, "{name}: model mismatch: {} struct {index}", stringify!($name))?;
                                writeln!(o, "  Actual: {actual:?}")?;
                                writeln!(o, "Expected: {var:?}")?;
                                break 'outer;
                            }
                        }
                    }

                    let duration = Instant::now().duration_since(start);
                    writeln!(o, " {duration:?}")?;
                }
            })*
        };
    }

    macro_rules! test {
        ($base:ident, $buf:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {{
            fuzz!($base $(, $name, $ty, $size_hint)*);
            size!($base $(, $name, $ty, $size_hint)*);
            run!($base $(, $name, $ty, $size_hint)*);
        }};
    }

    macro_rules! build {
        ($($name:ident, $ty:ty, $num:expr, $size_hint:expr),* $(,)?) => {
            $(
                let $name = generate::<$ty>(&mut rng, $num);
            )*

            musli_tests::feature_matrix!(test $(, $name, $ty, $size_hint)*);
        }
    }

    musli_tests::types!(build);

    if !size_sets.is_empty() {
        let mut footnotes = HashMap::new();

        footnotes.insert("musli_json", "[^incomplete]");
        footnotes.insert("rkyv", "[^incomplete]");
        footnotes.insert("serde_bitcode", "[^i128]");
        footnotes.insert("serde_cbor", "[^i128]");
        footnotes.insert("serde_dlhn", "[^i128]");
        footnotes.insert("serde_json", "[^incomplete]");
        footnotes.insert("derive_bitcode", "[^i128]");

        let mut columns = Vec::new();
        let mut rows = BTreeSet::new();

        macro_rules! build_column {
            ($($name:ident, $ty:ty, $num:expr, $size_hint:expr),*) => {
                $(columns.push(stringify!($name));)*
            };
        }

        musli_tests::types!(build_column);

        let mut index = HashMap::<_, SizeSet>::new();

        for set in size_sets {
            rows.insert(set.framework);
            let replaced = index.insert((set.suite, set.framework), set);
            assert!(replaced.is_none());
        }

        write!(o, "| **framework** |")?;

        for suite in &columns {
            write!(o, " **{suite}** |")?;
        }

        writeln!(o)?;
        write!(o, "| - |")?;

        for _ in &columns {
            write!(o, " - |")?;
        }

        writeln!(o)?;

        for framework in rows {
            let footnote = footnotes.get(framework).copied().unwrap_or_default();
            write!(o, "| {framework}{footnote} |")?;

            for suite in &columns {
                let Some(mut set) = index
                    .remove(&(suite, framework))
                    .filter(|s| !s.samples.is_empty())
                else {
                    write!(o, " - |")?;
                    continue;
                };

                let len = set.samples.len() as f64;

                set.samples.sort();
                let mean = set.samples.iter().sum::<i64>() as f64 / len;

                let (Some(mn), Some(mx)) = (set.samples.first(), set.samples.last()) else {
                    write!(o, " - |")?;
                    continue;
                };

                let ss = set.samples.iter().map(|s| (*s as f64 - mean).powf(2.0));
                let stddev = (ss.sum::<f64>() / len).sqrt();

                write!(o, " <a title=\"samples: {len}, min: {mn}, max: {mx}, stddev: {stddev}\">{mean:.2} ± {stddev:.2}</a> |")?;
            }

            writeln!(o)?;
        }
    }

    Ok(())
}
