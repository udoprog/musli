use std::io;
use std::io::Write;
use std::time::Instant;

use musli_tests::models::{Allocated, Generate, LargeStruct, Primitives};
use musli_tests::utils;
use rand::prelude::*;

musli_tests::miri! {
    const ELEMENTS: usize = 100, 2;
    const PRIMITIVE: usize = 10000, 2;
    const ALLOCATED: usize = 1000, 2;
    const LARGE: usize = 30, 2;
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

    let primitives = generate::<Primitives>(&mut rng, ELEMENTS);
    let alloc = generate::<Allocated>(&mut rng, ELEMENTS);
    let large = generate::<LargeStruct>(&mut rng, ELEMENTS);

    #[allow(unused)]
    macro_rules! run {
        ($base:ident, $var:ident, $ty:ty, $iter:expr) => {{
            let name = concat!(stringify!($base), "/", stringify!($var));

            if condition(name) {
                write!(o, "{name}: ...")?;
                o.flush()?;
                let start = Instant::now();

                for _ in 0..$iter {
                    for &(index, ref var) in &$var {
                        let out = utils::$base::encode(var);
                        let actual = utils::$base::decode::<$ty>(&out);
                        assert_eq!(actual, *var, "{name}: {} struct {index}", stringify!($var));
                    }
                }

                let duration = Instant::now().duration_since(start);
                writeln!(o, "{duration:?}")?;
            }
        }};
    }

    #[allow(unused)]
    macro_rules! test {
        ($base:ident) => {{
            run!($base, primitives, Primitives, PRIMITIVE);
            run!($base, alloc, Allocated, ALLOCATED);
            run!($base, large, LargeStruct, LARGE);
        }};
    }

    musli_tests::feature_matrix!(test);
    Ok(())
}
