use std::time::Instant;

use criterion::black_box;
use musli::{Decode, Encode};
use musli_wire::{Fixed, FixedLength, WireEncoding};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[musli(packed)]
struct SmallStruct {
    x: f32,
    y: f32,
}

const ENCODING: WireEncoding<Fixed, FixedLength> = WireEncoding::new()
    .with_fixed_integers()
    .with_fixed_lengths();

#[inline(never)]
fn encode<T>(expected: &T) -> Vec<u8>
where
    T: Encode,
{
    ENCODING.to_vec(expected).unwrap()
}

#[inline(never)]
fn decode<'de, T>(data: &'de [u8]) -> T
where
    T: Decode<'de>,
{
    ENCODING.decode(data).unwrap()
}

fn main() {
    let start = Instant::now();

    let data = encode(&SmallStruct { x: 1.0, y: 1.0 });

    for _ in 0..100000000 {
        let output = decode::<SmallStruct>(&data[..]);
        black_box(output);
    }

    dbg!("decode", Instant::now().duration_since(start));
}
