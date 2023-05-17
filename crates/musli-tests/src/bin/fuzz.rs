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

    macro_rules! run {
        ($base:ident $(, $name:ident, $ty:ty)*) => {
            $({
                let name = concat!(stringify!($base), "/", stringify!($name));

                if condition(name) {
                    write!(o, "{name}: ")?;
                    o.flush()?;
                    let start = Instant::now();

                    let step = iter / 10;

                    for n in 0..iter {
                        if n % step == 0 {
                            write!(o, ".")?;
                            o.flush()?;
                        }

                        for &(index, ref var) in &$name {
                            let out = utils::$base::encode(var);
                            let actual = utils::$base::decode::<$ty>(&out);
                            assert_eq!(actual, *var, "{name}: {} struct {index}", stringify!($name));
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
