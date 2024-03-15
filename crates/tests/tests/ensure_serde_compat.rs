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

macro_rules! tester {
    ($tester:ident, $module:ident $(,)?) => {
        #[track_caller]
        fn $tester<T>()
        where
            T: Eq + fmt::Debug + Generate + Encode + DecodeOwned + Serialize + DeserializeOwned,
        {
            let mut rng = tests::rng_with_seed(RNG_SEED);
            let value1 = T::generate(&mut rng);

            let bytes = $module::to_vec(&value1).expect("Encode musli");
            let serde = EncodeSerde(&value1);
            let bytes2 = $module::to_vec(&serde).expect("Encode serde");

            assert_eq!(&bytes, &bytes2);

            let value2: T = $module::from_slice(&bytes2).expect("Decode musli");
            assert_eq!(value1, value2);

            let DecodeSerde(value3) = $module::from_slice(&bytes).expect("Decode serde");
            assert_eq!(value1, value3);
        }
    };
}

tester!(musli_storage_rt, musli_storage);
tester!(musli_wire_rt, musli_wire);
tester!(musli_descriptive_rt, musli_descriptive);

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
enum Enum {
    #[musli(rename = "Empty")]
    Empty,
    #[musli(rename = "Tuple")]
    Tuple(u32, u32),
    #[musli(rename = "Struct")]
    Struct { a: u32, b: u32 },
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
struct Struct {
    #[musli(rename = "a")]
    a: u32,
    /// TODO: Change to `String` to break.
    #[musli(rename = "b")]
    b: u64,
    #[musli(rename = "enum_")]
    enum_: Enum,
}

#[test]
fn musli_storage() {
    musli_storage_rt::<u32>();
    musli_storage_rt::<HashMap<String, u32>>();
    musli_storage_rt::<Enum>();
    musli_storage_rt::<Struct>();
}

#[test]
fn musli_wire() {
    musli_wire_rt::<u32>();
    musli_wire_rt::<HashMap<String, u32>>();
    musli_wire_rt::<Enum>();
    musli_wire_rt::<Struct>();
}

#[test]
fn musli_descriptive() {
    musli_descriptive_rt::<u32>();
    musli_descriptive_rt::<HashMap<String, u32>>();
    musli_descriptive_rt::<Enum>();
    musli_descriptive_rt::<Struct>();
}
