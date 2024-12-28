use std::env;
use std::fmt;
use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::mem::align_of;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use tests::generate;
use tests::models;
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

tests::miri! {
    const ITER: usize = 10000, 2;
    const LARGE_STRUCTS: usize = 10, 2;
    const PRIMITIVES: usize = 500, 2;
    const PRIMITIVES_PACKED: usize = 500, 2;
    const MEDIUM_ENUMS: usize = 500, 2;
    const ALLOCATED: usize = 100, 2;
    const MESHES: usize = 10, 2;
}

fn main() -> Result<()> {
    let root = env::var_os("CARGO_MANIFEST_DIR")
        .map(|path| PathBuf::from(path).join("..").join(".."))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut it = std::env::args().skip(1);

    let mut iter = ITER;
    let mut random = false;
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
                    " --iter <count>  - Perform the <count> number of iterations when fuzzing, (default: {})", ITER);
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
                    "Note: Running this utility under miri reduces the range of these constants."
                );
                println!();
                println!("  ITER = {ITER}");
                println!("  LARGE_STRUCTS = {LARGE_STRUCTS}");
                println!("  PRIMITIVES = {PRIMITIVES}");
                println!("  PRIMITIVES_PACKED = {PRIMITIVES_PACKED}");
                println!("  MEDIUM_ENUMS = {MEDIUM_ENUMS}");
                println!("  ALLOCATED = {ALLOCATED}");
                println!("  generate::STRING_RANGE = {:?}", generate::STRING_RANGE);
                println!("  generate::MAP_RANGE = {:?}", generate::MAP_RANGE);
                println!("  generate::VEC_RANGE = {:?}", generate::VEC_RANGE);
                println!(
                    "  models::PRIMITIVES_RANGE = {:?}",
                    models::PRIMITIVES_RANGE
                );
                println!("  models::MEDIUM_RANGE = {:?}", models::MEDIUM_RANGE);
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

        ($framework:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
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
                            save_decode(&mut o, &root, verbose, bytes.as_slice(), format_args!("{n}_{}", stringify!($framework)))?;
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

        ($framework:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
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

                    for var in &$name {
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
        ($framework:ident, $name:ident, $ty:ty, $size_hint:expr) => {{
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

                        for (index, var) in $name.iter().enumerate() {
                            let mut state = buf.state();
                            state.reset($size_hint, var);

                            let mut out = match state.encode(var) {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during encode: {error}")?;
                                    break 'outer;
                                }
                            };

                            if let Some(bytes) = out.as_bytes() {
                                if save {
                                    save_decode(&mut o, &root, verbose, bytes, format_args!("{index}_{}", stringify!($framework)))?;
                                }
                            }

                            let actual = match out.decode::<$ty>() {
                                Ok(value) => value,
                                Err(error) => {
                                    write!(o, "E")?;
                                    writeln!(o)?;
                                    writeln!(o, "{index}: error during decode: {error}")?;

                                    if let Some(bytes) = out.as_bytes() {
                                        let path = root.join("target").join(format!("{}_error.bin", stringify!($framework)));
                                        fs::write(&path, bytes).with_context(|| path.display().to_string())?;
                                        writeln!(o, "{index}: failing structure written to {}", path.display())?;
                                    }

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
            }}
        }};
    }

    let mut rng = tests::rng_with_seed(seed);

    macro_rules! build {
        ($name:ident, $ty:ty, $num:expr, $size_hint:expr) => {{
            let $name = rng.next_vector::<$ty>($num);
            if random {
                tests::feature_matrix!(random, $name, $ty, $size_hint);
            }

            if size {
                tests::feature_matrix!(size, $name, $ty, $size_hint);
            }

            if !random && !size {
                tests::feature_matrix!(run, $name, $ty, $size_hint);
            }
        }};
    }

    tests::types!(build);

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

fn save_decode<W>(
    o: &mut W,
    root: &Path,
    verbose: bool,
    bytes: &[u8],
    name: impl fmt::Display,
) -> Result<()>
where
    W: ?Sized + Write,
{
    let path = root.join("target").join(format!("{}_decode.bin", name));

    if verbose {
        writeln!(o, "Saving: {}", path.display())?;
    }

    fs::write(&path, bytes).with_context(|| path.display().to_string())?;
    Ok(())
}
