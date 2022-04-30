use std::collections::HashMap;

use musli::{Decode, Encode};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
pub struct SmallStruct {
    a: u32,
    b: u64,
    c: u128,
    d: f32,
    e: f64,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
pub enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[musli(transparent)]
    Variant2(u128),
    #[musli(transparent)]
    Variant3(u64),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[musli(default_field_tag = "name")]
pub struct LargeStruct {
    elements: Vec<SmallStruct>,
    medium: Vec<MediumEnum>,
    map: HashMap<u32, u64>,
}

pub fn generate_small_struct(rng: &mut StdRng) -> SmallStruct {
    SmallStruct {
        a: rng.gen(),
        b: rng.gen(),
        c: rng.gen(),
        d: rng.gen(),
        e: rng.gen(),
    }
}

pub fn generate_string(rng: &mut StdRng) -> String {
    format!("Hello {}", rng.gen_range(100000..500000))
}

pub fn generate_medium_enum(rng: &mut StdRng) -> MediumEnum {
    match rng.gen_range(0..=2) {
        0 => MediumEnum::Variant1(generate_string(rng)),
        1 => MediumEnum::Variant2(rng.gen()),
        _ => MediumEnum::Variant3(rng.gen()),
    }
}

pub fn generate_large_struct(rng: &mut StdRng) -> LargeStruct {
    let mut elements = Vec::new();

    for _ in 0..rng.gen_range(100..500) {
        elements.push(generate_small_struct(rng));
    }

    let mut medium = Vec::new();

    for _ in 0..rng.gen_range(200..500) {
        medium.push(generate_medium_enum(rng));
    }

    let mut map = HashMap::new();

    for _ in 0..342 {
        map.insert(rng.gen(), rng.gen());
    }

    LargeStruct {
        elements,
        medium,
        map,
    }
}
