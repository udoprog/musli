use std::env;
use std::fs;
#[allow(unused)]
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
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
    const PRIMITIVES_PACKED: usize = 500, 2;
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
    let root =
        env::var_os("CARGO_MANIFEST_DIR").map(|path| PathBuf::from(path).join("..").join(".."));

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

        ($framework:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                musli_tests::if_supported! {
                    $framework, $name, {

                    let name = concat!(stringify!($framework), "/", stringify!($name), "/random");

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

                            match utils::$framework::decode::<$ty>(&bytes) {
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
                }}
            })*
        };
    }

    #[allow(unused_mut)]
    let mut size_sets = Vec::<SizeSet>::new();

    macro_rules! size {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($framework:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                musli_tests::if_supported! {
                    $framework, $name, {
                    let name = concat!(stringify!($framework), "/", stringify!($name), "/size");

                    if size && condition(name) {
                        let mut buf = utils::$framework::new();

                        let mut set = SizeSet {
                            framework: stringify!($framework),
                            suite: stringify!($name),
                            samples: Vec::new(),
                        };

                        for var in &$name {
                            buf.with(|mut state| {
                                state.reset($size_hint, var);

                                match state.encode(var) {
                                    Ok(value) => {
                                        set.samples.push(value.len() as i64);
                                    }
                                    Err(error) => {
                                        writeln!(o, "{name}: error during encode: {error}")?;
                                    }
                                }

                                Ok::<_, anyhow::Error>(())
                            })?;
                        }

                        size_sets.push(set);
                    }
                }}
            })*
        };
    }

    macro_rules! run {
        ($framework:ident $(, $name:ident, $ty:ty, $size_hint:expr)*) => {
            $({
                musli_tests::if_supported! {
                    $framework, $name, {
                    let name = concat!(stringify!($framework), "/", stringify!($name));

                    if (!random && !size) && condition(name) {
                        write!(o, "{name}: ")?;
                        o.flush()?;
                        let start = Instant::now();
                        let step = iter / 10;

                        let mut buf = utils::$framework::new();

                        'outer:
                        for n in 0..iter {
                            if step == 0 || n % step == 0 {
                                write!(o, ".")?;
                                o.flush()?;
                            }

                            for (index, var) in $name.iter().enumerate() {
                                let break_outer = buf.with(|mut state| {
                                    state.reset($size_hint, var);

                                    let out = match state.encode(var) {
                                        Ok(value) => value,
                                        Err(error) => {
                                            write!(o, "E")?;
                                            writeln!(o)?;
                                            writeln!(o, "{index}: error during encode: {error}")?;
                                            return Ok(true);
                                        }
                                    };

                                    let actual = match out.decode::<$ty>() {
                                        Ok(value) => value,
                                        Err(error) => {
                                            write!(o, "E")?;
                                            writeln!(o)?;
                                            writeln!(o, "{index}: error during decode: {error}")?;

                                            if let (Some(root), Some(bytes)) = (&root, out.as_bytes()) {
                                                let path = root.join("target").join(format!("{}_error.bin", stringify!($framework)));
                                                fs::write(&path, bytes).with_context(|| path.display().to_string())?;
                                                writeln!(o, "{index}: failing structure written to {}", path.display())?;
                                            }

                                            return Ok(true);
                                        }
                                    };

                                    if actual != *var {
                                        write!(o, "C")?;
                                        writeln!(o)?;
                                        writeln!(o, "{name}: model mismatch: {} struct {index}", stringify!($name))?;
                                        writeln!(o, "  Actual: {actual:?}")?;
                                        writeln!(o, "Expected: {var:?}")?;
                                        return Ok(true);
                                    }

                                    Ok::<_, anyhow::Error>(false)
                                })?;

                                if break_outer {
                                    break 'outer;
                                }
                            }
                        }

                        let duration = Instant::now().duration_since(start);
                        writeln!(o, " {duration:?}")?;
                    }
                }}
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
        for SizeSet {
            suite,
            framework,
            samples,
        } in size_sets
        {
            writeln!(
                o,
                "{{\"suite\":\"{suite}\",\"framework\":\"{framework}\",\"samples\":{samples:?}}}"
            )?;
        }
    }

    Ok(())
}
