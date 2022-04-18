use std::collections::HashMap;
use std::time::Instant;

use criterion::black_box;
use musli::{Decode, Encode};
use musli_wire::{Fixed, FixedLength, WireEncoding};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(packed)]
struct SmallStruct {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[musli(transparent)]
    Variant2(f64),
    #[musli(transparent)]
    Variant3(u64),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(packed)]
struct BigStruct {
    elements: Vec<SmallStruct>,
    values: Vec<MediumEnum>,
    map: HashMap<u32, u32>,
}

fn generate_big_struct() -> BigStruct {
    use rand::prelude::*;

    let mut rng = StdRng::seed_from_u64(123412327832);

    let mut elements = Vec::new();
    let mut values = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(SmallStruct {
            x: rng.gen(),
            y: rng.gen(),
        });
    }

    for _ in 0..rng.gen_range(100usize..500) {
        values.push(match rng.gen_range(0..=2) {
            0 => MediumEnum::Variant1(format!("Hello {}", rng.gen_range(100000..500000))),
            1 => MediumEnum::Variant2(rng.gen()),
            _ => MediumEnum::Variant3(rng.gen()),
        });
    }

    let mut map = HashMap::new();

    for _ in 0..342 {
        map.insert(rng.gen(), rng.gen());
    }

    BigStruct {
        elements,
        values,
        map,
    }
}

#[inline(never)]
fn encode_bincode<T>(expected: &T) -> Vec<u8>
where
    T: Serialize,
{
    bincode::serialize(expected).unwrap()
}

#[inline(never)]
fn decode_bincode<'de, T>(data: &'de [u8]) -> T
where
    T: Deserialize<'de>,
{
    bincode::deserialize(data).unwrap()
}

const ENCODING: WireEncoding<Fixed, FixedLength> = WireEncoding::new()
    .with_fixed_integers()
    .with_fixed_lengths();

#[inline(never)]
fn encode_musli<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    ENCODING.to_vec(expected).unwrap()
}

#[inline(never)]
fn decode_musli<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    ENCODING.decode(data).unwrap()
}

fn main() {
    macro_rules! test {
        ($st:ident, $encode:ident, $decode:ident, $iter:expr) => {
            let start = Instant::now();

            for _ in 0..$iter {
                let data = $encode(&$st);
                let output = $decode::<BigStruct>(&data[..]);
                black_box(output);
            }

            dbg!(Instant::now().duration_since(start));
        };
    }

    let mut it = std::env::args().skip(1);

    let big_struct = generate_big_struct();

    match it.next().as_deref() {
        Some("bincode") => {
            println!("bincode");
            test!(big_struct, encode_bincode, decode_bincode, 1_000_000);
        }
        _ => {
            println!("musli");
            test!(big_struct, encode_musli, decode_musli, 1_000_000);
        }
    }
}
