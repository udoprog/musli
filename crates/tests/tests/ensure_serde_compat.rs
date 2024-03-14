//! Tests that ensure that serde compatibility encodes and decodes correctly.

/// Default random seed to use.
pub const RNG_SEED: u64 = 2818281828459045235;

use std::collections::HashMap;
use std::fmt;

use musli::de::DecodeOwned;
use musli::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tests::generate::Generate;

#[derive(Encode)]
#[musli(transparent)]
struct EncodeSerde<'a, T>(#[musli(with = musli_serde)] &'a T)
where
    T: Serialize;

#[derive(Decode)]
#[musli(transparent)]
struct DecodeSerde<T>(#[musli(with = musli_serde)] T)
where
    T: DeserializeOwned;

fn musli_storage_serde_roundtrip<T>()
where
    T: Eq + fmt::Debug + Generate + Encode + DecodeOwned + Serialize + DeserializeOwned,
{
    let mut rng = tests::rng_with_seed(RNG_SEED);
    let value1 = T::generate(&mut rng);

    let bytes = musli_storage::to_vec(&value1).expect("Encode musli");
    let pairs = EncodeSerde(&value1);
    let bytes2 = musli_storage::to_vec(&pairs).expect("Encode serde");

    assert_eq!(&bytes, &bytes2);

    let value2: T = musli_storage::from_slice(&bytes2).expect("Decode musli");
    assert_eq!(value1, value2);

    let DecodeSerde(value3) = musli_storage::from_slice(&bytes).expect("Decode serde");
    assert_eq!(value1, value3);
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
enum Enum {
    Empty,
    Tuple(u32, String),
    Struct { a: u32, b: String },
}

#[test]
fn map_pairs() {
    // Primitive numbers.
    musli_storage_serde_roundtrip::<u32>();
    // Test MapPairs.
    musli_storage_serde_roundtrip::<HashMap<String, u32>>();
    // Test Enums.
    musli_storage_serde_roundtrip::<Enum>();
}
