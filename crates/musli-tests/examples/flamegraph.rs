use std::io::Write;
use std::time::Instant;

use criterion::black_box;
use musli_tests::models::{Generate, LargeStruct};
use rand::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut o = std::io::stdout();

    macro_rules! test {
        ($st:ident, $base:ident, $iter:expr) => {
            write!(o, "{}: ", stringify!($base))?;

            let start = Instant::now();
            let step = $iter / 100;

            for n in 0..$iter {
                if n % step == 0 {
                    write!(o, ".")?;
                    o.flush()?;
                }

                let data = musli_tests::utils::$base::encode::<LargeStruct>(&$st);
                let data = black_box(data);
                let output = musli_tests::utils::$base::decode::<LargeStruct>(&data);
                black_box(output);
            }

            writeln!(o)?;
            writeln!(o, "{:?}", Instant::now().duration_since(start))?;
        };
    }

    let mut it = std::env::args().skip(1);

    let mut rng = StdRng::seed_from_u64(123412327832);
    let large_struct = rng.generate();

    let value = it.next();

    macro_rules! branch {
        ($base:ident) => {
            if let Some(stringify!($base)) = value.as_deref() {
                test!(large_struct, $base, 1_000_000);
                return Ok(());
            }
        };
    }

    macro_rules! infos {
        ($base:ident) => {
            writeln!(o, "- {}", stringify!($base))?;
        };
    }

    musli_tests::feature_matrix!(branch);

    writeln!(o, "Available modes:")?;
    musli_tests::feature_matrix!(infos);
    Ok(())
}
