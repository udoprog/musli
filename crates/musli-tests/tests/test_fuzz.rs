#![cfg(not(miri))]

use std::time::Instant;

use rand::prelude::*;

const SMALL_ITERATIONS: usize = 100_000;
const LARGE_ITERATIONS: usize = 1_000;

#[test]
fn test_fuzz() {
    let mut rng = StdRng::seed_from_u64(123412327832);

    macro_rules! test {
        ($base:ident) => {{
            let start = Instant::now();

            for _ in 0..SMALL_ITERATIONS {
                let small_struct = musli_tests::models::generate_small_struct(&mut rng);

                let out = musli_tests::utils::$base::encode(&small_struct);
                let actual =
                    musli_tests::utils::$base::decode::<musli_tests::models::SmallStruct>(&out);
                assert_eq!(actual, small_struct);
            }

            for _ in 0..LARGE_ITERATIONS {
                let large_struct = musli_tests::models::generate_large_struct(&mut rng);

                let out = musli_tests::utils::$base::encode(&large_struct);
                let actual =
                    musli_tests::utils::$base::decode::<musli_tests::models::LargeStruct>(&out);
                assert_eq!(actual, large_struct);
            }

            println!(
                "{}: {:?}",
                stringify!($base),
                Instant::now().duration_since(start)
            );
        }};
    }

    test!(musli_json);
    test!(musli_storage);
    test!(musli_wire);
    test!(musli_value);
}
