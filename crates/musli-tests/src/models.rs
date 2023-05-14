#[cfg(feature = "std")]
use std::collections::HashMap;

use alloc::string::String;
use alloc::vec::Vec;

use musli::{Decode, Encode};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_name = "name")]
pub struct Primitives {
    boolean: bool,
    character: char,
    unsigned8: u8,
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    #[cfg(any(model_128, model_all))]
    unsigned128: u128,
    signed8: i8,
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(any(model_128, model_all))]
    signed128: i128,
    unsignedsize: usize,
    signedsize: isize,
    #[cfg(any(model_floats, model_all))]
    float32: f32,
    #[cfg(any(model_floats, model_all))]
    float64: f64,
    string: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
pub enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[cfg(any(model_128, model_all))]
    #[musli(transparent)]
    Variant2(u128),
    #[musli(transparent)]
    Variant3(u64),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_name = "name")]
pub struct LargeStruct {
    elements: Vec<Primitives>,
    medium: Vec<MediumEnum>,
    #[cfg(feature = "std")]
    map: HashMap<String, u64>,
}

pub fn generate_primitives(rng: &mut StdRng) -> Primitives {
    Primitives {
        boolean: rng.gen(),
        character: rng.gen(),
        unsigned8: rng.gen(),
        unsigned16: rng.gen(),
        unsigned32: rng.gen(),
        unsigned64: rng.gen(),
        #[cfg(any(model_128, model_all))]
        unsigned128: rng.gen(),
        signed8: rng.gen(),
        signed16: rng.gen(),
        signed32: rng.gen(),
        signed64: rng.gen(),
        #[cfg(any(model_128, model_all))]
        signed128: rng.gen(),
        unsignedsize: rng.gen(),
        signedsize: rng.gen(),
        #[cfg(any(model_floats, model_all))]
        float32: rng.gen(),
        #[cfg(any(model_floats, model_all))]
        float64: rng.gen(),
        string: generate_string(rng),
        bytes: generate_bytes(rng),
    }
}

pub fn generate_string(rng: &mut StdRng) -> String {
    let mut string = String::new();

    for _ in 0..rng.gen_range(0..256) {
        string.push(rng.gen());
    }

    string
}

pub fn generate_bytes(rng: &mut StdRng) -> Vec<u8> {
    let mut bytes = Vec::new();

    for _ in 0..rng.gen_range(0..256) {
        bytes.push(rng.gen());
    }

    bytes
}

pub fn generate_medium_enum(rng: &mut StdRng) -> MediumEnum {
    match rng.gen_range(0..=2) {
        0 => MediumEnum::Variant1(generate_string(rng)),
        #[cfg(any(model_128, model_all))]
        1 => MediumEnum::Variant2(rng.gen()),
        _ => MediumEnum::Variant3(rng.gen()),
    }
}

pub fn generate_large_struct(rng: &mut StdRng) -> LargeStruct {
    let mut elements = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(generate_primitives(rng));
    }

    let mut medium = Vec::new();

    for _ in 0..rng.gen_range(200..500) {
        medium.push(generate_medium_enum(rng));
    }

    LargeStruct {
        elements,
        medium,
        #[cfg(feature = "std")]
        map: {
            let mut map = HashMap::new();

            for _ in 0..342 {
                map.insert(generate_string(rng), rng.gen());
            }

            map
        },
    }
}
