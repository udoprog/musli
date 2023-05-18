#![cfg_attr(feature = "bitcode", allow(clippy::assign_op_pattern))]

mod generate;

#[cfg(feature = "std")]
use std::collections::HashMap;

use core::ops::Range;

#[cfg(feature = "model_cstring")]
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

pub use self::generate::Generate;
pub use rand::prelude::*;

miri! {
    const PRIMITIVES_RANGE: Range<usize> = 10..100, 1..3;
    const MEDIUM_RANGE: Range<usize> = 10..100, 1..3;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
pub struct Primitives {
    boolean: bool,
    character: char,
    unsigned8: u8,
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    #[cfg(feature = "model_128")]
    unsigned128: u128,
    signed8: i8,
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(feature = "model_128")]
    signed128: i128,
    #[cfg(feature = "model_usize")]
    unsignedsize: usize,
    #[cfg(feature = "model_usize")]
    signedsize: isize,
    #[cfg(feature = "model_float")]
    float32: f32,
    #[cfg(feature = "model_float")]
    float64: f64,
}

#[cfg(feature = "rkyv")]
impl PartialEq<Primitives> for &ArchivedPrimitives {
    #[inline]
    fn eq(&self, other: &Primitives) -> bool {
        *other == **self
    }
}

impl Generate<Primitives> for StdRng {
    fn generate(&mut self) -> Primitives {
        Primitives {
            boolean: self.generate(),
            character: self.generate(),
            unsigned8: self.generate(),
            unsigned16: self.generate(),
            unsigned32: self.generate(),
            unsigned64: self.generate(),
            #[cfg(feature = "model_128")]
            unsigned128: self.generate(),
            signed8: self.generate(),
            signed16: self.generate(),
            signed32: self.generate(),
            signed64: self.generate(),
            #[cfg(feature = "model_128")]
            signed128: self.generate(),
            #[cfg(feature = "model_usize")]
            unsignedsize: self.generate(),
            #[cfg(feature = "model_usize")]
            signedsize: self.generate(),
            #[cfg(feature = "model_float")]
            float32: self.generate(),
            #[cfg(feature = "model_float")]
            float64: self.generate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
pub struct Allocated {
    string: String,
    bytes: Vec<u8>,
    #[cfg(feature = "model_cstring")]
    c_string: CString,
}

#[cfg(feature = "rkyv")]
impl PartialEq<Allocated> for &ArchivedAllocated {
    #[inline]
    fn eq(&self, other: &Allocated) -> bool {
        *other == **self
    }
}

impl Generate<Allocated> for StdRng {
    fn generate(&mut self) -> Allocated {
        Allocated {
            string: self.generate(),
            bytes: self.generate(),
            #[cfg(feature = "model_cstring")]
            c_string: self.generate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
pub struct Tuples {
    u0: (),
    u1: (bool,),
    u2: (bool, u8),
    u3: (bool, u8, u32),
    u4: (bool, u8, u32, u64),
    #[cfg(feature = "model_float")]
    u5: (bool, u8, u32, u64, f32),
    #[cfg(feature = "model_float")]
    u6: (bool, u8, u32, u64, f32, f64),
    i0: (),
    i1: (bool,),
    i2: (bool, i8),
    i3: (bool, i8, i32),
    i4: (bool, i8, i32, i64),
    #[cfg(feature = "model_float")]
    i5: (bool, i8, i32, i64, f32),
    #[cfg(feature = "model_float")]
    i6: (bool, i8, i32, i64, f32, f64),
}

#[cfg(feature = "rkyv")]
impl PartialEq<Tuples> for &ArchivedTuples {
    #[inline]
    fn eq(&self, other: &Tuples) -> bool {
        *other == **self
    }
}

impl Generate<Tuples> for StdRng {
    fn generate(&mut self) -> Tuples {
        Tuples {
            u0: self.generate(),
            u1: self.generate(),
            u2: self.generate(),
            u3: self.generate(),
            u4: self.generate(),
            #[cfg(feature = "model_float")]
            u5: self.generate(),
            #[cfg(feature = "model_float")]
            u6: self.generate(),
            i0: self.generate(),
            i1: self.generate(),
            i2: self.generate(),
            i3: self.generate(),
            i4: self.generate(),
            #[cfg(feature = "model_float")]
            i5: self.generate(),
            #[cfg(feature = "model_float")]
            i6: self.generate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed))]
pub enum MediumEnum {
    #[cfg_attr(feature = "musli", musli(transparent))]
    StringVariant(String),
    #[cfg_attr(feature = "musli", musli(transparent))]
    NumbereVariant(u64),
    EmptyTupleVariant(),
    NamedEmptyVariant {},
    NamedVariant {
        a: u32,
        primitives: Primitives,
        b: u64,
    },
    UnnamedVariant,
}

#[cfg(feature = "rkyv")]
impl PartialEq<MediumEnum> for &ArchivedMediumEnum {
    #[inline]
    fn eq(&self, other: &MediumEnum) -> bool {
        *other == **self
    }
}

impl Generate<MediumEnum> for StdRng {
    fn generate(&mut self) -> MediumEnum {
        match self.gen_range(0..=5) {
            0 => MediumEnum::StringVariant(self.generate()),
            1 => MediumEnum::NumbereVariant(self.generate()),
            2 => MediumEnum::EmptyTupleVariant(),
            3 => MediumEnum::NamedEmptyVariant {},
            4 => MediumEnum::NamedVariant {
                a: self.generate(),
                primitives: self.generate(),
                b: self.generate(),
            },
            _ => MediumEnum::UnnamedVariant,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
pub struct LargeStruct {
    primitives: Vec<Primitives>,
    #[cfg(all(feature = "model_vec", feature = "model_tuple"))]
    tuples: Vec<(Tuples, Tuples)>,
    medium_vec: Vec<MediumEnum>,
    #[cfg(feature = "model_map_string_key")]
    medium_map: HashMap<String, MediumEnum>,
    #[cfg(feature = "model_map_string_key")]
    string_keys: HashMap<String, u64>,
    #[cfg(feature = "model_map")]
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

impl Generate<LargeStruct> for StdRng {
    fn generate(&mut self) -> LargeStruct {
        LargeStruct {
            primitives: self.generate_range(PRIMITIVES_RANGE),
            #[cfg(all(feature = "model_vec", feature = "model_tuple"))]
            tuples: self.generate_range(PRIMITIVES_RANGE),
            medium_vec: self.generate_range(MEDIUM_RANGE),
            #[cfg(feature = "model_map_string_key")]
            medium_map: self.generate_range(MEDIUM_RANGE),
            #[cfg(feature = "model_map_string_key")]
            string_keys: self.generate(),
            #[cfg(feature = "model_map")]
            number_keys: self.generate(),
            number_vec: self.generate(),
        }
    }
}
