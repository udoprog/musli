use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use musli_tests::models::*;
use musli_tests::utils;

musli_tests::miri! {
    const ITER: usize = 10000, 2;
    const LARGE_STRUCTS: usize = 10, 2;
    const PRIMITIVES: usize = 500, 2;
    const MEDIUM_ENUMS: usize = 500, 2;
    const ALLOCATED: usize = 100, 2;
}

fn generate<T>(rng: &mut StdRng, count: usize) -> Vec<(usize, T)>
where
    StdRng: Generate<T>,
{
    let mut out = Vec::with_capacity(count);

    for index in 0..count {
        out.push((index, rng.generate()));
    }

    out
}

fn main() -> Result<()> {
    let mut rng = musli_tests::rng();

    let mut it = std::env::args().skip(1);

    let mut iter = ITER;
    let mut random = false;
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
            random_bytes.push(rng.generate_range(0..256));
        }
    }

    macro_rules! fuzz {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($base:ident $(, $name:ident, $ty:ty)*) => {
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

    macro_rules! run {
        ($base:ident $(, $name:ident, $ty:ty)*) => {
            $({
                let name = concat!(stringify!($base), "/", stringify!($name));

                if !random && condition(name) {
                    write!(o, "{name}: ")?;
                    o.flush()?;
                    let start = Instant::now();
                    let step = iter / 10;

                    'outer:
                    for n in 0..iter {
                        if step == 0 || n % step == 0 {
                            write!(o, ".")?;
                            o.flush()?;
                        }

                        for &(index, ref var) in &$name {
                            let out = match utils::$base::encode(var) {
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
                                writeln!(o, "actual: {actual:?}")?;
                                writeln!(o, "expected: {var:?}")?;
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
        ($base:ident $(, $name:ident, $ty:ty)*) => {{
            fuzz!($base $(, $name, $ty)*);
            run!($base $(, $name, $ty)*);
        }};
    }

    macro_rules! build {
        ($($name:ident, $ty:ty, $num:expr),*) => {
            $(
                let $name = generate::<$ty>(&mut rng, $num);
            )*

            musli_tests::feature_matrix!(test $(, $name, $ty)*);
        }
    }

    musli_tests::types!(build);
    Ok(())
}
