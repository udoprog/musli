#![cfg_attr(feature = "bitcode", allow(clippy::assign_op_pattern))]

use core::ops::Range;

#[cfg(all(feature = "std", not(any(feature = "rkyv", feature = "dlhn"))))]
use std::collections::HashMap;

use alloc::string::String;
use alloc::vec::Vec;

use musli::{Decode, Encode};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::mode::Packed;

macro_rules! ranges {
    ($($(#[$($meta:meta)*])* const $ident:ident: Range<usize> = $range:expr, $miri:expr;)*) => {
        $(
            $(#[$($meta)*])*
            #[cfg(miri)]
            const $ident: Range<usize> = $miri;
            $(#[$($meta)*])*
            #[cfg(not(miri))]
            const $ident: Range<usize> = $range;
        )*
    }
}

ranges! {
    const STRING_RANGE: Range<usize> = 0..256, 0..16;
    const MAP_RANGE: Range<usize> = 100..500, 1..3;
    const PRIMITIVES_RANGE: Range<usize> = 100..500, 1..3;
    const MEDIUM_RANGE: Range<usize> = 200..500, 1..3;
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[musli(mode = Packed, packed)]
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
    #[cfg(not(feature = "rkyv"))]
    unsignedsize: usize,
    #[cfg(not(feature = "rkyv"))]
    signedsize: isize,
    #[cfg(any(model_floats, model_all))]
    float32: f32,
    #[cfg(any(model_floats, model_all))]
    float64: f64,
    string: String,
    bytes: Vec<u8>,
}

#[cfg(feature = "rkyv")]
impl PartialEq<Primitives> for &ArchivedPrimitives {
    #[inline]
    fn eq(&self, other: &Primitives) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[musli(mode = Packed)]
pub enum MediumEnum {
    #[musli(transparent)]
    Variant1(String),
    #[cfg(any(model_128, model_all))]
    #[musli(transparent)]
    Variant2(u128),
    #[musli(transparent)]
    Variant3(u64),
    Variant4 {
        a: u32,
        primitives: Primitives,
        b: u64,
    },
    Variant5,
}

#[cfg(feature = "rkyv")]
impl PartialEq<MediumEnum> for &ArchivedMediumEnum {
    #[inline]
    fn eq(&self, other: &MediumEnum) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[musli(mode = Packed, packed)]
pub struct LargeStruct {
    elements: Vec<Primitives>,
    medium: Vec<MediumEnum>,
    #[cfg(all(feature = "std", not(feature = "rkyv")))]
    string_keys: HashMap<String, u64>,
    #[cfg(all(feature = "std", not(feature = "dlhn")))]
    number_keys: HashMap<u32, u64>,
    number_vec: Vec<(u32, u64)>,
}

#[cfg(feature = "rkyv")]
impl PartialEq<LargeStruct> for &ArchivedLargeStruct {
    #[inline]
    fn eq(&self, other: &LargeStruct) -> bool {
        *other == **self
    }
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
        #[cfg(not(feature = "rkyv"))]
        unsignedsize: rng.gen(),
        #[cfg(not(feature = "rkyv"))]
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

    for _ in 0..rng.gen_range(STRING_RANGE) {
        string.push(rng.gen());
    }

    string
}

pub fn generate_bytes(rng: &mut StdRng) -> Vec<u8> {
    let mut bytes = Vec::new();

    for _ in 0..rng.gen_range(STRING_RANGE) {
        bytes.push(rng.gen());
    }

    bytes
}

pub fn generate_medium_enum(rng: &mut StdRng) -> MediumEnum {
    match rng.gen_range(0..=4) {
        0 => MediumEnum::Variant1(generate_string(rng)),
        #[cfg(any(model_128, model_all))]
        1 => MediumEnum::Variant2(rng.gen()),
        2 => MediumEnum::Variant3(rng.gen()),
        3 => MediumEnum::Variant4 {
            a: rng.gen(),
            primitives: generate_primitives(rng),
            b: rng.gen(),
        },
        _ => MediumEnum::Variant5,
    }
}

pub fn generate_large_struct(rng: &mut StdRng) -> LargeStruct {
    let mut elements = Vec::new();

    for _ in 0..rng.gen_range(PRIMITIVES_RANGE) {
        elements.push(generate_primitives(rng));
    }

    let mut medium = Vec::new();

    for _ in 0..rng.gen_range(MEDIUM_RANGE) {
        medium.push(generate_medium_enum(rng));
    }

    LargeStruct {
        elements,
        medium,
        #[cfg(all(feature = "std", not(feature = "rkyv")))]
        string_keys: {
            let mut map = HashMap::new();

            for _ in 0..rng.gen_range(MAP_RANGE) {
                map.insert(generate_string(rng), rng.gen());
            }

            map
        },
        #[cfg(all(feature = "std", not(feature = "dlhn")))]
        number_keys: {
            let mut map = HashMap::new();

            for _ in 0..rng.gen_range(MAP_RANGE) {
                map.insert(rng.gen(), rng.gen());
            }

            map
        },
        number_vec: {
            let mut vec = Vec::new();

            for _ in 0..rng.gen_range(MAP_RANGE) {
                vec.push((rng.gen(), rng.gen()));
            }

            vec
        },
    }
}
