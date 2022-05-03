#![cfg(not(miri))]

use std::time::Instant;

use musli_tests::models;
use musli_tests::utils;
use rand::prelude::*;

const PRIMITIVES_ITERATIONS: usize = 100;
const LARGE_ITERATIONS: usize = 100;

#[test]
fn test_fuzz() {
    let mut rng = StdRng::seed_from_u64(123412327832);

    let mut primitives = Vec::new();
    let mut large = Vec::new();

    for index in 0..PRIMITIVES_ITERATIONS {
        primitives.push((index, models::generate_primitives(&mut rng)));
    }

    for index in 0..LARGE_ITERATIONS {
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
            let name = stringify!($base);
            let start = Instant::now();

            for &(index, ref primitives) in &primitives$([$range])* {
                let out = utils::$base::encode(primitives);
                let actual = utils::$base::decode::<models::Primitives>(&out);
                assert_eq!(actual, *primitives, "{name}: primitives struct {index}");
            }

            let duration = Instant::now().duration_since(start);
            println!("{name}: {duration:?}: primitives");
        }};
    }

    #[allow(unused)]
    macro_rules! test_large {
        ($base:ident $(, $range:expr)?) => {{
            let name = stringify!($base);
            let start = Instant::now();

            for &(index, ref large_struct) in &large$([$range])* {
                let out = utils::$base::encode(large_struct);
                let actual = utils::$base::decode::<models::LargeStruct>(&out);
                assert_eq!(actual, *large_struct, "{name}: large struct {index}");
            }

            let duration = Instant::now().duration_since(start);
            println!("{name}: {duration:?}: large");
        }};
    }

    test!(musli_json);
    test!(serde_json);
    test!(musli_storage);
    test!(musli_wire);
    test!(musli_value);
}
