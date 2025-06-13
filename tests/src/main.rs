#![no_std]

extern crate std;

extern crate alloc;

use std::env;
use std::fmt;
use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::mem::align_of;
use std::path::{Path, PathBuf};
use std::prelude::v1::*;
use std::time::Instant;
use std::{format, println};

use anyhow::{bail, Context, Result};
use tests::models::*;
use tests::utils;
use tests::AlignedBuf;
use tests::Generate;

const ALIGNMENT: usize = align_of::<u128>();

struct SizeSet {
    framework: &'static str,
    suite: &'static str,
    samples: Vec<i64>,
}

tests::options! {
    pub unsafe fn init_constants();
    pub(crate) fn enumerate_constants();
    static ITER: usize = 1000, 2;
    static LARGE: usize = 10, 2;
    static PRIMITIVES: usize = 500, 2;
    static PACKED: usize = 500, 2;
    static FULL_ENUM: usize = 500, 2;
    static ALLOCATED: usize = 100, 2;
    static MESHES: usize = 10, 2;
}

fn main() -> Result<()> {
    // SAFETY: These are only initialized *once* at the beginning of the program.
    unsafe {
        init_constants();
        tests::init_statics();
    }

    let target = env::var_os("CARGO_TARGET_DIR")
        .map(|path| PathBuf::from(path).join("musli"))
        .unwrap_or_else(|| PathBuf::from("target").join("musli"));

    let mut it = std::env::args().skip(1);

    let mut iter = ITER.get();
    let mut random = false;
    let mut all = false;
    let mut size = false;
    let mut filter = Vec::new();
    let mut seed = tests::RNG_SEED;
    let mut alignment = ALIGNMENT;
    let mut verbose = false;
    let mut save = false;

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--iter" => {
                iter = it
                    .next()
                    .context("missing argument for `--iter`")?
                    .parse()
                    .context("bad argument to --iter")?;
            }
            "--seed" => {
                seed = it
                    .next()
                    .context("missing argument for `--seed`")?
                    .parse()
                    .context("bad argument to --seed")?;
            }
            "--align" => {
                alignment = it
                    .next()
                    .context("missing argument for `--align`")?
                    .parse()
                    .context("bad argument to --align")?;
            }
            "--random" => {
                random = true;
            }
            "--all" => {
                all = true;
            }
            "--size" => {
                size = true;
            }
            "--verbose" => {
                verbose = true;
            }
            "--save" => {
                save = true;
            }
            "-h" | "--help" => {
                println!("Available options:");
                println!(
                    " --iter <count>  - Run the given number of iterations (default: {})",
                    ITER.get()
                );
                println!(" --random        - Feed each framework randomly generated.");
                println!(
                    " --size          - Construct random data structures and print their sizes."
                );
                println!(
                    " --seed <seed>   - Use the specified random seed (default: {}).",
                    tests::RNG_SEED
                );
                println!(
                    " --align <align> - Use the specified random seed (default: {}).",
                    ALIGNMENT
                );
                println!(
                    " --save          - Before decoding something, save the bytes to the `target` folder. Use `--verbose` to figure out where."
                );
                println!();
                println!(
                    "Variables in use that can be overrided through environment variables, note that running under miri reduces their value:"
                );
                println!();

                let f: fn(&str, &dyn core::fmt::Debug) = |key, value| {
                    println!("  {key} = {value:?}");
                };

                enumerate_constants(f);
                tests::enumerate_statics(f);
                return Ok(());
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

    let mut random_bytes: Vec<AlignedBuf> = Vec::new();

    random_bytes.push(AlignedBuf::new(alignment));

    if random {
        let mut rng = tests::rng_with_seed(seed);

        for _ in 0..iter {
            let mut alignable = AlignedBuf::new(alignment);
            let bytes: Vec<u8> = Generate::generate_range(&mut rng, 0..256);
            alignable.extend_from_slice(&bytes);
            random_bytes.push(alignable);
        }
    }

    macro_rules! random {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($framework:ident, $values:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
            tests::if_supported! {
                $framework, $name, {

                let name = concat!(stringify!($framework), "/", stringify!($name), "/random");

                if utils::$framework::is_enabled() && condition(name) {
                    write!(o, "{name}: ")?;
                    o.flush()?;
                    let start = Instant::now();

                    let step = random_bytes.len() / 10;
                    let mut errors = Vec::new();

                    for (n, bytes) in random_bytes.iter().enumerate() {
                        if (step == 0 || n % step == 0) {
                            if !errors.is_empty() {
                                write!(o, "E")?;
                            } else {
                                write!(o, ".")?;
                            }

                            if verbose {
                                for error in errors.drain(..) {
                                    writeln!(o)?;
                                    writeln!(o, "{}", error)?;
                                    // errors are expected, so don't log them as failures.
                                    black_box(error);
                                }
                            } else {
                                // errors are expected, so don't log them as failures.
                                black_box(errors.drain(..).collect::<Vec<_>>());
                            }

                            o.flush()?;
                        }

                        if save {
                            save_file(
                                &mut o,
                                &target,
                                verbose,
                                bytes.as_slice(),
                                format_args!("{}_{}_{n}_decode", stringify!($framework), stringify!($name))
                            )?;
                        }

                        match utils::$framework::decode::<$ty>(bytes.as_slice()) {
                            Ok(value) => {
                                // values *can* randomly occur.
                                black_box(value);
                            }
                            Err(error) => {
                                errors.push(error);
                            }
                        }
                    }

                    let duration = Instant::now().duration_since(start);
                    writeln!(o, " {duration:?}")?;
                }
            }}
        }};
    }

    let mut size_sets = Vec::<SizeSet>::new();

    macro_rules! size {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {};

        ($framework:ident, $values:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
            tests::if_supported! {
                $framework, $name, {
                let name = concat!(stringify!($framework), "/", stringify!($name), "/size");

                if utils::$framework::is_enabled() && condition(name) {
                    let mut buf = utils::$framework::new();

                    let mut set = SizeSet {
                        framework: stringify!($framework),
                        suite: stringify!($name),
                        samples: Vec::new(),
                    };

                    for var in $values.iter() {
                        let mut state = buf.state();
                        state.reset($size_hint, var);

                        match state.encode(var) {
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
            }}
        }};
    }

    macro_rules! run {
        ($framework:ident, $values:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
            tests::if_supported! {
                $framework, $name, {
                let name = concat!(stringify!($framework), "/", stringify!($name));

                if utils::$framework::is_enabled() && condition(name) {
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

                        for (index, var) in $values.iter().enumerate() {
                            let mut state = buf.state();
                            state.reset($size_hint, var);

                            let result = state.encode(var);

                            let mut out = match result {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during encode: {error}")?;
                                    break 'outer;
                                }
                            };

                            let write_bytes = |o: &mut dyn Write, bytes: Option<&[u8]>, suffix: &str| {
                                if let Some(bytes) = bytes {
                                    let path = save_file(
                                        o,
                                        &target,
                                        verbose,
                                        bytes,
                                        format_args!("{}_{}_{n}_{index}_{suffix}", stringify!($framework), stringify!($name))
                                    )?;
                                    writeln!(o, "{index}: failing structure written to {}", path.display())?;
                                }

                                Ok::<_, anyhow::Error>(())
                            };

                            if save {
                                write_bytes(&mut o, out.as_bytes(), "decode")?;
                            }

                            let result = out.decode::<$ty>();

                            let actual = match result {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during decode: {error}")?;
                                    write_bytes(&mut o, out.as_bytes(), "error")?;
                                    break 'outer;
                                }
                            };

                            if actual != *var {
                                write!(o, "C")?;
                                writeln!(o)?;
                                writeln!(o, "{name}: model mismatch: {} struct {index}", stringify!($name))?;
                                writeln!(o, "  Actual: {actual:?}")?;
                                writeln!(o, "Expected: {var:?}")?;
                                write_bytes(&mut o, out.as_bytes(), "error")?;
                                break 'outer;
                            }
                        }
                    }

                    let duration = Instant::now().duration_since(start);
                    writeln!(o, " {duration:?}")?;
                }
            }}
        }};
    }

    let mut rng = tests::rng_with_seed(seed);

    macro_rules! build {
        ($name:ident, $ty:ty, $num:expr, $size_hint:expr) => {{
            let values = rng.next_vector::<$ty>($num.get());

            if random {
                tests::feature_matrix!(random, values, $name, $ty, $size_hint);
            }

            if size {
                tests::feature_matrix!(size, values, $name, $ty, $size_hint);
            }

            if !random && !size && !all {
                tests::feature_matrix!(run, values, $name, $ty, $size_hint);
            }
        }};
    }

    tests::types!(build);

    if all {
        macro_rules! all {
            ($name:ident, $ty:ty) => {
                let values: Vec<$ty> = rng.next_vector::<$ty>(ITER.get());
                tests::feature_matrix!(run, values, $name, $ty, 0);
            };
        }

        tests::basic_types!(all);
    }

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

fn save_file<W>(
    o: &mut W,
    target: &Path,
    verbose: bool,
    bytes: &[u8],
    name: impl fmt::Display,
) -> Result<PathBuf>
where
    W: ?Sized + Write,
{
    let path = target.join(format!("{}.bin", name));

    if verbose {
        writeln!(o, "Saving: {}", path.display())?;
    }

    if let Some(dir) = path.parent() {
        if !dir.is_dir() {
            fs::create_dir_all(dir).with_context(|| path.display().to_string())?;
        }
    }

    fs::write(&path, bytes).with_context(|| path.display().to_string())?;
    Ok(path)
}
