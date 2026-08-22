//! Targeted benchmarks for the shared number parser, [`musli::number`].
//!
//! The `json` benchmark measures whole documents against other frameworks. This
//! one aims at the number parser on its own: every document here is nothing but
//! numbers, and the groups differ only in how those numbers are written, so the
//! cost of each shape the parser distinguishes shows up separately.
//!
//! The four entry points into the parser are covered:
//!
//! * `parse_unsigned` and `parse_signed`, through decoding into an integer.
//! * `parse_float`, through decoding into a float.
//! * `parse_any`, through decoding into a [`Value`], which is handed a number
//!   without being told which type it is wanted as.
//! * `skip`, through decoding a struct whose remaining fields are unknown.
//!
//! Both syntaxes are covered as well. JSON documents are read as [RFC 8259],
//! and the JSONB documents additionally exercise the JSON5 forms which SQLite
//! stores in its `INT5` and `FLOAT5` elements, hexadecimals among them.
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p tests --bench number --features test,parse-full
//! ```
//!
//! Two things are worth knowing before reading a number off this.
//!
//! The parser is only reachable through a decoder, so every figure carries
//! whatever the decoder costs on the way in. The `vs-rust` group measures that
//! directly: a single digit is almost all decoder, so the distance between the
//! one digit figures and the longer ones is the part which is actually the
//! number.
//!
//! And the whole binary is one unit of code, so a change anywhere in it moves
//! where everything else lands. Groups which parse no numbers at all have been
//! seen to move by several percent that way, so anything under roughly ten
//! percent is worth confirming against a group the change cannot have touched
//! before believing it. Pinning the run to an idle core, with `taskset`, cuts
//! the rest of the noise down.
//!
//! [`Value`]: musli::value::Value
//! [RFC 8259]: https://datatracker.ietf.org/doc/html/rfc8259#section-6

use criterion::Criterion;

#[cfg(all(
    feature = "musli-json",
    feature = "musli-sqlite-jsonb",
    feature = "musli-value",
    feature = "parse-full"
))]
mod bench {
    use std::fmt::Write;
    use std::hint::black_box;

    use criterion::measurement::WallTime;
    use criterion::{BenchmarkGroup, Criterion, Throughput};
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};

    /// How many numbers each document holds. Every group reports per-number
    /// times, so this only decides how much of the harness each measurement
    /// amortizes over.
    const COUNT: usize = 4096;

    /// Element types of the JSONB format, which are the four the number parser
    /// is reached through. See `musli::sqlite_jsonb`.
    const INT: u8 = 3;
    const INT5: u8 = 4;
    const FLOAT: u8 = 5;
    const FLOAT5: u8 = 6;
    const ARRAY: u8 = 11;

    /// Decode a JSON document of numbers, checking first that it decodes at all.
    macro_rules! json {
        ($g:expr, $name:expr, $ty:ty, $data:expr) => {{
            let data: &str = $data;
            musli::json::from_str::<$ty>(data).unwrap();

            $g.bench_function($name, |b| {
                b.iter(|| musli::json::from_str::<$ty>(black_box(data)).unwrap())
            });
        }};
    }

    /// Decode a JSONB document of numbers, checking first that it decodes at
    /// all.
    macro_rules! jsonb {
        ($g:expr, $name:expr, $ty:ty, $data:expr) => {{
            let data: &[u8] = $data;
            musli::sqlite_jsonb::from_slice::<$ty>(data).unwrap();

            $g.bench_function($name, |b| {
                b.iter(|| musli::sqlite_jsonb::from_slice::<$ty>(black_box(data)).unwrap())
            });
        }};
    }

    /// A struct which keeps one field and leaves the rest to be skipped.
    #[derive(musli::Decode)]
    #[allow(dead_code)]
    struct Keep {
        keep: u32,
    }

    /// Write out the numbers in `values` as a JSON array.
    fn array<I>(values: I) -> String
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut out = String::from("[");

        for (n, value) in values.into_iter().enumerate() {
            if n > 0 {
                out.push(',');
            }

            out.push_str(value.as_ref());
        }

        out.push(']');
        out
    }

    /// Write the header of a JSONB element of the given `kind` carrying a
    /// payload of `len` bytes.
    fn header(out: &mut Vec<u8>, kind: u8, len: usize) {
        if len <= 11 {
            out.push(((len as u8) << 4) | kind);
        } else if let Ok(len) = u8::try_from(len) {
            out.push(0xc0 | kind);
            out.push(len);
        } else if let Ok(len) = u16::try_from(len) {
            out.push(0xd0 | kind);
            out.extend_from_slice(&len.to_be_bytes());
        } else {
            out.push(0xe0 | kind);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
    }

    /// Write out the numbers in `values` as a JSONB array of `kind` elements.
    fn jsonb_array<I>(kind: u8, values: I) -> Vec<u8>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut payload = Vec::new();

        for value in values {
            let value = value.as_ref();
            header(&mut payload, kind, value.len());
            payload.extend_from_slice(value.as_bytes());
        }

        let mut out = Vec::new();
        header(&mut out, ARRAY, payload.len());
        out.extend_from_slice(&payload);
        out
    }

    /// Build `COUNT` numbers by formatting each index through `f`.
    fn numbers<F>(rng: &mut StdRng, mut f: F) -> Vec<String>
    where
        F: FnMut(&mut String, &mut StdRng),
    {
        (0..COUNT)
            .map(|_| {
                let mut out = String::new();
                f(&mut out, rng);
                out
            })
            .collect()
    }

    /// Open a group which reports the time it takes to decode one number.
    fn group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
        let mut g = c.benchmark_group(name);
        g.throughput(Throughput::Elements(COUNT as u64));
        g
    }

    /// Integers written as nothing but their digits, which is the path the vast
    /// majority of numbers take, at each width the accumulator is instantiated
    /// for.
    fn plain(c: &mut Criterion, rng: &mut StdRng) {
        let mut g = group(c, "plain");

        let digits1 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(0u8..10)).unwrap()
        });
        json!(g, "u8/1-digit", Vec<u8>, &array(&digits1));

        let digits5 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(10_000u32..100_000)).unwrap()
        });
        json!(g, "u32/5-digits", Vec<u32>, &array(&digits5));

        let digits19 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(u64::MAX / 2..u64::MAX)).unwrap()
        });
        json!(g, "u64/19-digits", Vec<u64>, &array(&digits19));

        let digits38 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(u128::MAX / 2..u128::MAX)).unwrap()
        });
        json!(g, "u128/38-digits", Vec<u128>, &array(&digits38));

        // The same digits read as a signed integer, half of them negative, so
        // the extra sign handling and the range check are visible against the
        // unsigned run above.
        let signed = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(i64::MIN / 2..i64::MAX / 2)).unwrap()
        });
        json!(g, "i64/mixed-sign", Vec<i64>, &array(&signed));

        g.finish();
    }

    /// Whole numbers written with a fraction or an exponent, which is what
    /// takes the parser through `tail` and `combine` rather than stopping at the
    /// digits.
    ///
    /// The baseline is the same values written plainly, so the difference is
    /// the cost of the tail alone.
    fn tail(c: &mut Criterion, rng: &mut StdRng) {
        let mut g = group(c, "tail");

        let base: Vec<u64> = (0..COUNT)
            .map(|_| rng.random_range(100_000u64..1_000_000))
            .collect();

        let plain: Vec<String> = base.iter().map(|v| v.to_string()).collect();
        json!(g, "u64/baseline", Vec<u64>, &array(&plain));

        let point: Vec<String> = base.iter().map(|v| format!("{v}.0")).collect();
        json!(g, "u64/point-zero", Vec<u64>, &array(&point));

        // A long run of zeros behind the point, which the fraction scanner walks
        // over without accumulating.
        let zeros: Vec<String> = base.iter().map(|v| format!("{v}.00000000")).collect();
        json!(g, "u64/trailing-zeros", Vec<u64>, &array(&zeros));

        // Scaled up by an exponent, which multiplies the digits by a power of
        // ten.
        let up: Vec<String> = base.iter().map(|v| format!("{v}e3")).collect();
        json!(g, "u64/exponent-up", Vec<u64>, &array(&up));

        // Scaled back down again, which divides and checks that nothing is left
        // behind the point.
        let down: Vec<String> = base.iter().map(|v| format!("{v}000e-3")).collect();
        json!(g, "u64/exponent-down", Vec<u64>, &array(&down));

        // A fraction which the exponent scales into a whole number, which is the
        // path where both halves of `combine` do work.
        let both: Vec<String> = base
            .iter()
            .map(|v| {
                let text = v.to_string();
                let (head, rest) = text.split_at(1);
                format!("{head}.{rest}e5")
            })
            .collect();
        json!(g, "u64/fraction-exponent", Vec<u64>, &array(&both));

        g.finish();
    }

    /// Floats, which the parser only picks apart far enough to hand the digits
    /// to the decimal to float conversion.
    fn float(c: &mut Criterion, rng: &mut StdRng) {
        let mut g = group(c, "float");

        let short = numbers(rng, |o, rng| {
            write!(o, "{:.3}", rng.random_range(-1.0e3f64..1.0e3)).unwrap()
        });
        json!(g, "f64/short", Vec<f64>, &array(&short));

        let long = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(-1.0e6f64..1.0e6)).unwrap()
        });
        json!(g, "f64/roundtrip", Vec<f64>, &array(&long));

        let exponent = numbers(rng, |o, rng| {
            write!(o, "{:e}", rng.random_range(-1.0e6f64..1.0e6)).unwrap()
        });
        json!(g, "f64/exponent", Vec<f64>, &array(&exponent));

        // Integers asked for as floats, which is the one shape where the number
        // is whole but the conversion still runs.
        let whole = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(0u32..1_000_000)).unwrap()
        });
        json!(g, "f64/whole", Vec<f64>, &array(&whole));
        json!(g, "f32/short", Vec<f32>, &array(&short));

        g.finish();
    }

    /// Numbers decoded without being told which type they are wanted as, which
    /// goes through `parse_any` and picks the narrowest type which holds the
    /// value.
    fn any(c: &mut Criterion, rng: &mut StdRng) {
        use musli::alloc::Global;
        use musli::value::Value;

        let mut g = group(c, "any");

        let small = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(0u32..1_000_000)).unwrap()
        });
        json!(g, "unsigned", Vec<Value<Global>>, &array(&small));

        // The same magnitudes carrying a sign, so the negation and the range
        // check are the only difference from the run above.
        let signed = numbers(rng, |o, rng| {
            write!(o, "-{}", rng.random_range(0u32..1_000_000)).unwrap()
        });
        json!(g, "signed", Vec<Value<Global>>, &array(&signed));

        // Numbers which are not whole, where `parse_any` runs the scanner, finds
        // the value is fractional and falls back to the float conversion.
        let fractional = numbers(rng, |o, rng| {
            write!(o, "{:.3}", rng.random_range(-1.0e3f64..1.0e3)).unwrap()
        });
        json!(g, "float", Vec<Value<Global>>, &array(&fractional));

        g.finish();
    }

    /// Numbers which are only measured, which is what an unknown field of an
    /// object costs.
    fn skip(c: &mut Criterion, rng: &mut StdRng) {
        let mut g = group(c, "skip");

        let objects = numbers(rng, |o, rng| {
            let keep = rng.random_range(0u32..1_000);
            let a = rng.random_range(0u64..u64::MAX);
            let b = rng.random_range(-1.0e6f64..1.0e6);
            write!(o, r#"{{"keep":{keep},"a":{a},"b":{b}}}"#).unwrap();
        });
        json!(g, "unknown-fields", Vec<Keep>, &array(&objects));

        g.finish();
    }

    /// The JSONB elements, which read the same digits as canonical JSON in
    /// `INT` and `FLOAT` and as JSON5 in `INT5` and `FLOAT5`. The JSON5 forms
    /// are the only way to reach the hexadecimal and leading sign handling.
    fn jsonb(c: &mut Criterion, rng: &mut StdRng) {
        let mut g = group(c, "jsonb");

        let base: Vec<u64> = (0..COUNT)
            .map(|_| rng.random_range(0u64..1_000_000_000))
            .collect();

        let decimal: Vec<String> = base.iter().map(|v| v.to_string()).collect();
        jsonb!(g, "int/decimal", Vec<u64>, &jsonb_array(INT, &decimal));
        jsonb!(g, "int5/decimal", Vec<u64>, &jsonb_array(INT5, &decimal));

        let hex: Vec<String> = base.iter().map(|v| format!("0x{v:x}")).collect();
        jsonb!(g, "int5/hex", Vec<u64>, &jsonb_array(INT5, &hex));

        let plus: Vec<String> = base.iter().map(|v| format!("+{v}")).collect();
        jsonb!(g, "int5/plus", Vec<u64>, &jsonb_array(INT5, &plus));

        let floats: Vec<String> = base.iter().map(|v| format!("{}.5", *v as f64)).collect();
        jsonb!(g, "float/decimal", Vec<f64>, &jsonb_array(FLOAT, &floats));

        // A leading point, which only JSON5 permits.
        let leading: Vec<String> = base.iter().map(|v| format!(".{v}")).collect();
        jsonb!(
            g,
            "float5/leading-point",
            Vec<f64>,
            &jsonb_array(FLOAT5, &leading)
        );

        // Hexadecimals asked for as floats, which is the one form the decimal to
        // float conversion does not understand and which the parser picks off
        // itself.
        jsonb!(g, "float5/hex", Vec<f64>, &jsonb_array(INT5, &hex));

        g.finish();
    }

    /// The same numbers read one at a time by the parser here and by the one in
    /// the standard library, which is what the module docs weigh the explicit
    /// scanner against.
    ///
    /// Both sides are handed one number per call over the same literals, so the
    /// only thing between them is the parsing. The parser here is reached
    /// through a whole JSON document of one number, so its figure additionally
    /// carries whatever setting up a decoder costs; the standard library is
    /// reached through [`str::parse`], which is [`dec2flt`] for the floats and
    /// an accumulator much like this one for the integers.
    ///
    /// [`dec2flt`]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
    fn against_rust(c: &mut Criterion, rng: &mut StdRng) {
        /// Read every literal in `values` both ways, checking first that the
        /// two agree on all of them.
        macro_rules! compare {
            ($g:expr, $name:expr, $ty:ty, $values:expr) => {{
                let values: &[String] = $values;

                for value in values {
                    let a = musli::json::from_str::<$ty>(value).unwrap();
                    let b = value.parse::<$ty>().unwrap();
                    assert_eq!(a, b, "{value}");
                }

                $g.bench_function(concat!($name, "/musli"), |b| {
                    b.iter(|| {
                        for value in black_box(values) {
                            black_box(musli::json::from_str::<$ty>(value).unwrap());
                        }
                    })
                });

                $g.bench_function(concat!($name, "/rust"), |b| {
                    b.iter(|| {
                        for value in black_box(values) {
                            black_box(value.parse::<$ty>().unwrap());
                        }
                    })
                });
            }};
        }

        let mut g = group(c, "vs-rust");

        // A single digit, where there is almost no number to parse, so the two
        // figures are mostly what each side costs before it reaches one. The
        // groups above are the ones which measure numbers of a real size.
        let digits1 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(0u8..10)).unwrap()
        });
        compare!(g, "u8/1-digit", u8, &digits1);

        let digits5 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(10_000u32..100_000)).unwrap()
        });
        compare!(g, "u32/5-digits", u32, &digits5);

        let digits19 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(u64::MAX / 2..u64::MAX)).unwrap()
        });
        compare!(g, "u64/19-digits", u64, &digits19);

        let digits38 = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(u128::MAX / 2..u128::MAX)).unwrap()
        });
        compare!(g, "u128/38-digits", u128, &digits38);

        let signed = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(i64::MIN / 2..i64::MAX / 2)).unwrap()
        });
        compare!(g, "i64/mixed-sign", i64, &signed);

        let short = numbers(rng, |o, rng| {
            write!(o, "{:.3}", rng.random_range(-1.0e3f64..1.0e3)).unwrap()
        });
        compare!(g, "f64/short", f64, &short);

        let long = numbers(rng, |o, rng| {
            write!(o, "{}", rng.random_range(-1.0e6f64..1.0e6)).unwrap()
        });
        compare!(g, "f64/roundtrip", f64, &long);

        let exponent = numbers(rng, |o, rng| {
            write!(o, "{:e}", rng.random_range(-1.0e6f64..1.0e6)).unwrap()
        });
        compare!(g, "f64/exponent", f64, &exponent);
        compare!(g, "f32/short", f32, &short);

        // Hexadecimals, which only the JSON5 syntax accepts and which the
        // standard library reads through `from_str_radix`.
        let base: Vec<u64> = (0..COUNT)
            .map(|_| rng.random_range(0u64..1_000_000_000))
            .collect();

        let hex: Vec<String> = base.iter().map(|v| format!("{v:x}")).collect();
        let document = jsonb_array(INT5, base.iter().map(|v| format!("0x{v:x}")));

        assert_eq!(
            musli::sqlite_jsonb::from_slice::<Vec<u64>>(&document).unwrap(),
            base
        );

        g.bench_function("u64/hex/musli", |b| {
            b.iter(|| musli::sqlite_jsonb::from_slice::<Vec<u64>>(black_box(&document)).unwrap())
        });

        g.bench_function("u64/hex/rust", |b| {
            b.iter(|| {
                for value in black_box(&hex) {
                    black_box(u64::from_str_radix(value, 16).unwrap());
                }
            })
        });

        g.finish();
    }

    pub(super) fn run(c: &mut Criterion) {
        let mut rng = StdRng::seed_from_u64(tests::RNG_SEED);

        plain(c, &mut rng);
        tail(c, &mut rng);
        float(c, &mut rng);
        any(c, &mut rng);
        skip(c, &mut rng);
        jsonb(c, &mut rng);
        against_rust(c, &mut rng);
    }
}

#[cfg(all(
    feature = "musli-json",
    feature = "musli-sqlite-jsonb",
    feature = "musli-value",
    feature = "parse-full"
))]
fn criterion_benchmark(c: &mut Criterion) {
    self::bench::run(c);
}

#[cfg(not(all(
    feature = "musli-json",
    feature = "musli-sqlite-jsonb",
    feature = "musli-value",
    feature = "parse-full"
)))]
fn criterion_benchmark(_: &mut Criterion) {
    eprintln!("Skipping: requires --features musli-json,musli-sqlite-jsonb,musli-value,parse-full");
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
