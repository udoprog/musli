//! Mutation fuzzing: encode a valid value, corrupt it, and make sure decoding
//! only ever errors rather than panicking.

use std::collections::HashMap;

use musli::alloc::Global;
use musli::value::Value;
use musli::{Decode, Encode};

/// Number of mutations to try per format. Miri is orders of magnitude slower,
/// so it only gets a token amount to keep the CI job manageable.
#[cfg(not(miri))]
const ITER: usize = 50000;
#[cfg(miri)]
const ITER: usize = 64;

#[derive(Debug, PartialEq, Encode, Decode)]
struct Inner {
    a: u32,
    b: String,
    c: Vec<u8>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum En {
    Unit,
    Newtype(u32),
    Tuple(u32, String),
    Struct { x: i64, y: Option<u8> },
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Outer {
    inner: Inner,
    list: Vec<En>,
    opt: Option<Inner>,
    tup: (u8, i16, f64),
    map: HashMap<String, u32>,
    ch: char,
    flag: bool,
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn sample() -> Outer {
    let mut map = HashMap::new();
    map.insert("key".to_string(), 7u32);

    Outer {
        inner: Inner {
            a: 5,
            b: "hello".into(),
            c: vec![1, 2, 3, 4],
        },
        list: vec![
            En::Unit,
            En::Newtype(3),
            En::Tuple(1, "t".into()),
            En::Struct { x: -2, y: Some(4) },
        ],
        opt: Some(Inner {
            a: 0,
            b: String::new(),
            c: Vec::new(),
        }),
        tup: (1, -2, 3.5),
        map,
        ch: '\u{1f600}',
        flag: true,
    }
}

fn mutate(rng: &mut Rng, original: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();

    for _ in 0..rng.below(6) + 1 {
        if bytes.is_empty() {
            bytes.push((rng.next() & 0xFF) as u8);
            continue;
        }

        let at = rng.below(bytes.len());

        match rng.below(4) {
            0 => bytes[at] ^= 1u8 << rng.below(8),
            1 => bytes[at] = (rng.next() & 0xFF) as u8,
            2 => bytes.truncate(at),
            _ => bytes.insert(at, (rng.next() & 0xFF) as u8),
        }
    }

    bytes
}

macro_rules! fuzz {
    ($module:ident, $seed:expr) => {{
        let value = sample();
        let original = musli::$module::to_vec(&value).unwrap();
        let mut rng = Rng($seed);

        for _ in 0..ITER {
            let bytes = mutate(&mut rng, &original);

            // Only errors or values are acceptable, never a panic.
            let _ = musli::$module::from_slice::<Outer>(&bytes);
            let _ = musli::$module::from_slice::<Inner>(&bytes);
            let _ = musli::$module::from_slice::<Vec<En>>(&bytes);
            let _ = musli::$module::from_slice::<u64>(&bytes);
            let _ = musli::$module::from_slice::<String>(&bytes);
        }
    }};
}

#[test]
fn fuzz_storage() {
    fuzz!(storage, 0x1111);
}

#[test]
fn fuzz_wire() {
    fuzz!(wire, 0x2222);
}

#[test]
fn fuzz_descriptive() {
    fuzz!(descriptive, 0x3333);
}

#[test]
fn fuzz_json() {
    fuzz!(json, 0x4444);
}

#[test]
fn fuzz_descriptive_into_value() {
    let value = sample();
    let original = musli::descriptive::to_vec(&value).unwrap();
    let mut rng = Rng(0x5555);

    for _ in 0..ITER {
        let bytes = mutate(&mut rng, &original);
        let _ = musli::descriptive::from_slice::<Value<Global>>(&bytes);
    }
}
