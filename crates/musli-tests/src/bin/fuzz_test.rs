use std::io;
use std::io::Write;
use std::time::Instant;

use musli_tests::models;
use musli_tests::utils;
use rand::prelude::*;

const PRIMITIVE_STRUCTS: usize = 100;
const LARGE_STRUCTS: usize = 100;

const PRIMITIVE_ITERATIONS: usize = 10_000;
const LARGE_ITERATIONS: usize = 30;

fn main() -> io::Result<()> {
    let filter = std::env::args().skip(1).collect::<Vec<_>>();

    let condition = move |name: &str| {
        if filter.is_empty() {
            return true;
        }

        filter.iter().all(|f| name.contains(f))
    };

    let stdout = std::io::stdout();
    let mut o = stdout.lock();

    let mut rng = StdRng::seed_from_u64(123412327832);

    let mut primitives = Vec::new();
    let mut large = Vec::new();

    for index in 0..PRIMITIVE_STRUCTS {
        primitives.push((index, models::generate_primitives(&mut rng)));
    }

    for index in 0..LARGE_STRUCTS {
        large.push((index, models::generate_large_struct(&mut rng)));
    }

    #[allow(unused)]
    macro_rules! test {
        ($base:ident $(, $range:expr)?) => {{
            test_primitives!($base $(, $range)*);
            test_large!($base $(, $range)*);
        }};
    }

    #[allow(unused)]
    macro_rules! test_primitives {
        ($base:ident $(, $range:expr)?) => {{
            let name = concat!(stringify!($base), "/primitives");

            if condition(name) {
                write!(o, "{name}: ... ")?;
                o.flush()?;

                let start = Instant::now();

                for _ in 0..PRIMITIVE_ITERATIONS {
                    for &(index, ref primitives) in &primitives$([$range])* {
                        let out = utils::$base::encode(primitives);
                        let actual = utils::$base::decode::<models::Primitives>(&out);
                        assert_eq!(actual, *primitives, "{name}: primitives struct {index}");
                    }
                }

                let duration = Instant::now().duration_since(start);
                writeln!(o, "{duration:?}")?;
            }
        }};
    }

    #[allow(unused)]
    macro_rules! test_large {
        ($base:ident $(, $range:expr)?) => {{
            let name = concat!(stringify!($base), "/large");

            if condition(name) {
                write!(o, "{name}: ... ")?;
                o.flush()?;

                let start = Instant::now();

                for _ in 0..LARGE_ITERATIONS {
                    for &(index, ref large_struct) in &large$([$range])* {
                        let out = utils::$base::encode(large_struct);
                        let actual = utils::$base::decode::<models::LargeStruct>(&out);
                        assert_eq!(actual, *large_struct, "{name}: large struct {index}");
                    }
                }

                let duration = Instant::now().duration_since(start);
                writeln!(o, "{duration:?}")?;
            }
        }};
    }

    test!(musli_json);
    test!(serde_json);
    test!(musli_storage);
    test!(musli_wire);
    test!(musli_value);
    Ok(())
}
