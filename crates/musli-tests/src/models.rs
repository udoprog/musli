#![cfg_attr(feature = "bitcode", allow(clippy::assign_op_pattern))]

mod generate;

#[cfg(any(feature = "model_map", feature = "model_map_string_key"))]
use std::collections::HashMap;

use core::ops::Range;

#[cfg(feature = "model_cstring")]
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "musli-zerocopy")]
use musli_zerocopy::ZeroCopy;

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
use musli_macros::Generate;
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

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli-zerocopy", derive(ZeroCopy))]
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
#[cfg_attr(feature = "musli-zerocopy", repr(C))]
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

impl PartialEq<Primitives> for &Primitives {
    #[inline]
    fn eq(&self, other: &Primitives) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Generate)]
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

impl PartialEq<Allocated> for &Allocated {
    #[inline]
    fn eq(&self, other: &Allocated) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
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

impl PartialEq<Tuples> for &Tuples {
    #[inline]
    fn eq(&self, other: &Tuples) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Generate)]
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
    NumberedVariant(u64),
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

impl PartialEq<MediumEnum> for &MediumEnum {
    #[inline]
    fn eq(&self, other: &MediumEnum) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Generate)]
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
    #[generate(range = PRIMITIVES_RANGE)]
    primitives: Vec<Primitives>,
    #[cfg(all(feature = "model_vec", feature = "model_tuple"))]
    #[generate(range = PRIMITIVES_RANGE)]
    tuples: Vec<(Tuples, Tuples)>,
    #[generate(range = MEDIUM_RANGE)]
    medium_vec: Vec<MediumEnum>,
    #[cfg(feature = "model_map_string_key")]
    #[generate(range = MEDIUM_RANGE)]
    medium_map: HashMap<String, MediumEnum>,
    #[cfg(feature = "model_map_string_key")]
    string_keys: HashMap<String, u64>,
    #[cfg(all(feature = "model_map", feature = "model_tuple"))]
    number_keys: HashMap<u32, u64>,
    #[cfg(feature = "model_tuple")]
    number_vec: Vec<(u32, u64)>,
}

#[cfg(feature = "rkyv")]
impl PartialEq<LargeStruct> for &ArchivedLargeStruct {
    #[inline]
    fn eq(&self, other: &LargeStruct) -> bool {
        *other == **self
    }
}

impl PartialEq<LargeStruct> for &LargeStruct {
    #[inline]
    fn eq(&self, other: &LargeStruct) -> bool {
        *other == **self
    }
}
