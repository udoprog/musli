#![no_std]

extern crate std;

extern crate alloc;

use std::env;
use std::fmt;
use std::format;
use std::fs;
use std::hint::black_box;
use std::io::{StdoutLock, Write};
use std::mem::align_of;
use std::mem::take;
use std::path::PathBuf;
use std::prelude::v1::*;
use std::time::Instant;

use anyhow::{Context as _, Result, bail};
use tests::AlignedBuf;
use tests::Generate;
use tests::models::*;
use tests::utils;

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

struct Ctxt<'a> {
    o: &'a mut StdoutLock<'static>,
    target: PathBuf,
    iter: usize,
    random: bool,
    all: bool,
    size: bool,
    filter: Vec<String>,
    seed: u64,
    alignment: usize,
    verbose: bool,
    save: bool,
    decode: Vec<(String, PathBuf)>,
}

impl Ctxt<'_> {
    fn condition(&self, name: &str) -> bool {
        if self.filter.is_empty() {
            return true;
        }

        self.filter.iter().all(|f| name.contains(f))
    }

    fn save_file(&mut self, bytes: &[u8], name: impl fmt::Display) -> Result<PathBuf> {
        let path = self.target.join(format!("{name}.bin"));

        if self.verbose {
            writeln!(self.o, "Saving: {}", path.display())?;
        }

        if let Some(dir) = path.parent()
            && !dir.is_dir()
        {
            fs::create_dir_all(dir).with_context(|| path.display().to_string())?;
        }

        fs::write(&path, bytes).with_context(|| path.display().to_string())?;
        Ok(path)
    }
}

impl Write for Ctxt<'_> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.o.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.o.flush()
    }
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

    let stdout = std::io::stdout();
    let mut _stdout = stdout.lock();

    let mut cx = Ctxt {
        o: &mut _stdout,
        target,
        iter: ITER.get(),
        random: false,
        all: false,
        size: false,
        filter: Vec::new(),
        seed: tests::RNG_SEED,
        alignment: ALIGNMENT,
        verbose: false,
        save: false,
        decode: Vec::new(),
    };

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--iter" => {
                cx.iter = it
                    .next()
                    .context("missing argument for `--iter`")?
                    .parse()
                    .context("bad argument to --iter")?;
            }
            "--seed" => {
                cx.seed = it
                    .next()
                    .context("missing argument for `--seed`")?
                    .parse()
                    .context("bad argument to --seed")?;
            }
            "--align" => {
                cx.alignment = it
                    .next()
                    .context("missing argument for `--align`")?
                    .parse()
                    .context("bad argument to --align")?;
            }
            "--random" => {
                cx.random = true;
            }
            "--all" => {
                cx.all = true;
            }
            "--size" => {
                cx.size = true;
            }
            "--decode" => {
                let id = it
                    .next()
                    .context("missing `id` argument for `--decode <id> <path>`")?;

                let path = it
                    .next()
                    .context("missing `path` argument for `--decode <id> <path>`")?;

                cx.decode.push((id.to_owned(), PathBuf::from(path)));
            }
            "--verbose" => {
                cx.verbose = true;
            }
            "--save" => {
                cx.save = true;
            }
            "-h" | "--help" => {
                writeln!(cx, "Available options:")?;
                writeln!(
                    cx,
                    " --iter <count>  - Run the given number of iterations (default: {})",
                    ITER.get()
                )?;
                writeln!(
                    cx,
                    " --random        - Feed each framework randomly generated."
                )?;
                writeln!(
                    cx,
                    " --size          - Construct random data structures and print their sizes."
                )?;
                writeln!(
                    cx,
                    " --seed <seed>   - Use the specified random seed (default: {}).",
                    tests::RNG_SEED
                )?;
                writeln!(
                    cx,
                    " --align <align> - Use the specified random seed (default: {ALIGNMENT})."
                )?;
                writeln!(
                    cx,
                    " --save          - Before decoding something, save the bytes to the `target` folder. Use `--verbose` to figure out where."
                )?;
                writeln!(cx)?;
                writeln!(
                    cx,
                    "Variables in use that can be overrided through environment variables, note that running under miri reduces their value:"
                )?;
                writeln!(cx)?;

                enumerate_constants(|key, value| writeln!(cx, "  {key} = {value:?}"))?;
                tests::enumerate_statics(|key, value| writeln!(cx, "  {key} = {value:?}"))?;
                return Ok(());
            }
            other if other.starts_with("--") => {
                bail!("Bad argument: {other}");
            }
            _ => {
                cx.filter.push(arg);
            }
        }
    }

    macro_rules! random {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {
        };

        ($framework:ident, $name:ident, $cx:ident, $random_bytes:ident, $ty:ty, $size_hint:expr) => {{
            tests::if_supported! {
                $framework, $name, {

                let name = concat!(stringify!($framework), "/", stringify!($name), "/random");

                if utils::$framework::is_enabled() && $cx.condition(name) {
                    write!($cx, "{name}: ")?;
                    $cx.flush()?;
                    let start = Instant::now();

                    let step = $random_bytes.len() / 10;
                    let mut errors = Vec::new();

                    for (n, bytes) in $random_bytes.iter().enumerate() {
                        if (step == 0 || n % step == 0) {
                            if !errors.is_empty() {
                                write!($cx, "E")?;
                            } else {
                                write!($cx, ".")?;
                            }

                            if $cx.verbose {
                                for error in errors.drain(..) {
                                    writeln!($cx)?;
                                    writeln!($cx, "{}", error)?;
                                    // errors are expected, so don't log them as failures.
                                    black_box(error);
                                }
                            } else {
                                // errors are expected, so don't log them as failures.
                                black_box(errors.drain(..).collect::<Vec<_>>());
                            }

                            $cx.flush()?;
                        }

                        if $cx.save {
                            $cx.save_file(
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
                    writeln!($cx, " {duration:?}")?;
                }
            }}
        }};
    }

    macro_rules! size {
        // musli value is not a bytes-oriented encoding.
        (musli_value $($tt:tt)*) => {};

        ($framework:ident, $values:ident, $name:ident, $cx:ident, $ty:ty, $size_hint:expr, $out:ident) => {{
            tests::if_supported! {
                $framework, $name, {
                let name = concat!(stringify!($framework), "/", stringify!($name), "/size");

                'bail: {
                    if utils::$framework::is_enabled() && $cx.condition(name) {
                        let mut buf = match utils::$framework::setup() {
                            Ok(buf) => buf,
                            Err(error) => {
                                writeln!($cx, "{name}: error during setup: {error}")?;
                                break 'bail;
                            }
                        };

                        let mut set = SizeSet {
                            framework: stringify!($framework),
                            suite: stringify!($name),
                            samples: Vec::new(),
                        };

                        for var in $values.iter() {
                            let mut state = buf.state();

                            if let Err(error) = state.reset($size_hint, var) {
                                writeln!($cx, "{name}: error during reset: {error}")?;
                                continue;
                            };

                            match state.encode(var) {
                                Ok(value) => {
                                    set.samples.push(value.len() as i64);
                                }
                                Err(error) => {
                                    writeln!($cx, "{name}: error during encode: {error}")?;
                                }
                            }
                        }

                        $out.push(set);
                    }
                }
            }}
        }};
    }

    macro_rules! run {
        ($framework:ident, $values:ident, $name:ident, $cx:ident, $ty:ty, $size_hint:expr, $compare:path) => {{
            tests::if_supported! {
                $framework, $name, {
                let name = concat!(stringify!($framework), "/", stringify!($name));

                if utils::$framework::is_enabled() && $cx.condition(name) {
                    write!($cx, "{name}: ")?;
                    $cx.flush()?;
                    let start = Instant::now();
                    let step = $cx.iter / 10;

                    let mut buf = match utils::$framework::setup() {
                        Ok(buf) => buf,
                        Err(error) => {
                            writeln!($cx, "E")?;
                            writeln!($cx)?;
                            writeln!($cx, "error during setup: {error}")?;
                            return Ok(());
                        }
                    };

                    'outer:
                    for n in 0..$cx.iter {
                        if step == 0 || n % step == 0 {
                            write!($cx, ".")?;
                            $cx.flush()?;
                        }

                        for (index, var) in $values.iter().enumerate() {
                            let mut state = buf.state();

                            if let Err(error) = state.reset($size_hint, var) {
                                write!($cx, "E")?;
                                writeln!($cx)?;
                                writeln!($cx, "{index}: error during reset: {error}")?;
                                break 'outer;
                            };

                            let result = state.encode(var);

                            let mut out = match result {
                                Ok(value) => value,
                                Err(error) => {
                                    write!($cx, "E")?;
                                    writeln!($cx)?;
                                    writeln!($cx, "{index}: error during encode: {error}")?;
                                    break 'outer;
                                }
                            };

                            let write_bytes = |cx: &mut Ctxt<'_>, bytes: Option<&[u8]>, suffix: &str| {
                                if let Some(bytes) = bytes {
                                    let path = cx.save_file(
                                        bytes,
                                        format_args!("{}_{}_{n}_{index}_{suffix}", stringify!($framework), stringify!($name))
                                    )?;
                                    writeln!(cx, "{index}: failing structure written to {}", path.display())?;
                                }

                                Ok::<_, anyhow::Error>(())
                            };

                            if $cx.save {
                                write_bytes($cx, out.as_bytes(), "decode")?;
                            }

                            let result = out.decode::<$ty>();

                            let actual = match result {
                                Ok(value) => value,
                                Err(error) => {
                                    write!($cx, "E")?;
                                    writeln!($cx)?;
                                    writeln!($cx, "{index}: error during decode: {error}")?;
                                    write_bytes($cx, out.as_bytes(), "error")?;
                                    break 'outer;
                                }
                            };

                            if !$compare(&actual, var) {
                                write!($cx, "C")?;
                                writeln!($cx)?;
                                writeln!($cx, "{name}: model mismatch: {} struct {index}", stringify!($name))?;
                                writeln!($cx, "  Actual: {actual:?}")?;
                                writeln!($cx, "Expected: {var:?}")?;
                                write_bytes($cx, out.as_bytes(), "error")?;
                                break 'outer;
                            }
                        }
                    }

                    let duration = Instant::now().duration_since(start);
                    writeln!($cx, " {duration:?}")?;
                }
            }}
        }};
    }

    let mut rng = tests::rng_with_seed(cx.seed);

    let mut size_sets = Vec::<SizeSet>::new();
    let other = !cx.random && !cx.size && !cx.all && cx.decode.is_empty();

    macro_rules! build {
        ($name:ident, $ty:ty, $num:expr, $size_hint:expr) => {{
            if cx.random {
                let mut random_bytes: Vec<AlignedBuf> = Vec::new();

                random_bytes.push(AlignedBuf::new(cx.alignment));

                if cx.random {
                    let mut rng = tests::rng_with_seed(cx.seed);

                    for _ in 0..cx.iter {
                        let mut alignable = AlignedBuf::new(cx.alignment);
                        let bytes: Vec<u8> = Generate::generate_range(&mut rng, 0..256);
                        alignable.extend_from_slice(&bytes);
                        random_bytes.push(alignable);
                    }
                }

                fn $name(cx: &mut Ctxt<'_>, random_bytes: &[AlignedBuf]) -> Result<()> {
                    tests::feature_matrix!(random, $name, cx, random_bytes, $ty, $size_hint);
                    Ok(())
                }

                $name(&mut cx, &random_bytes[..])?;
            }

            if cx.size || other {
                let values = rng.next_vector::<$ty>($num.get());

                if cx.size {
                    fn $name(
                        cx: &mut Ctxt<'_>,
                        values: &[$ty],
                        out: &mut Vec<SizeSet>,
                    ) -> Result<()> {
                        tests::feature_matrix!(size, values, $name, cx, $ty, $size_hint, out);
                        Ok(())
                    }

                    $name(&mut cx, &values[..], &mut size_sets)?;
                }

                if other {
                    fn $name(cx: &mut Ctxt<'_>, values: &[$ty]) -> Result<()> {
                        tests::feature_matrix!(
                            run,
                            values,
                            $name,
                            cx,
                            $ty,
                            $size_hint,
                            tests::partial_eq
                        );
                        Ok(())
                    }

                    $name(&mut cx, &values[..])?;
                }
            }
        }};
    }

    tests::types!(build);

    if cx.all {
        macro_rules! all {
            ($name:ident, $ty:ty, $compare:path) => {
                fn $name(cx: &mut Ctxt<'_>) -> Result<()> {
                    let mut rng = tests::rng_with_seed(cx.seed);

                    let values: Vec<$ty> = rng.next_vector::<$ty>(ITER.get());
                    tests::feature_matrix!(run, values, $name, cx, $ty, 0, $compare);
                    Ok(())
                }

                $name(&mut cx)?;
            };
        }

        tests::basic_types!(all);
    }

    for (what, path) in take(&mut cx.decode) {
        let contents = fs::read(&path).with_context(|| format!("{}", path.display()))?;

        macro_rules! decode_inner {
            ($ident:ident, $ty:ty, $compare:path, $framework:ident) => {
                if stringify!($ident) == what
                    && utils::$framework::is_enabled()
                    && cx.condition(stringify!($framework))
                {
                    match utils::$framework::decode::<$ty>(contents.as_slice()) {
                        Ok(value) => {
                            writeln!(cx, "{value:?}")?;
                        }
                        Err(error) => {
                            writeln!(cx, "error during decode: {error}")?;
                        }
                    }
                }
            };
        }

        macro_rules! decode {
            ($framework:ident) => {
                tests::if_supported! {
                    $framework, decode_bytes, {
                        tests::basic_types!(decode_inner, $framework);
                    }
                }
            };
        }

        tests::feature_matrix!(decode);
    }

    if !size_sets.is_empty() {
        for SizeSet {
            suite,
            framework,
            samples,
        } in size_sets
        {
            writeln!(
                cx,
                "{{\"suite\":\"{suite}\",\"framework\":\"{framework}\",\"samples\":{samples:?}}}"
            )?;
        }
    }

    Ok(())
}
