#![cfg_attr(feature = "bitcode", allow(clippy::assign_op_pattern))]

#[cfg(not(feature = "model-no-map"))]
use std::collections::HashMap;

use core::ops::Range;

#[cfg(not(feature = "model-no-cstring"))]
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "musli-zerocopy")]
use musli_zerocopy::ZeroCopy;

#[cfg(feature = "zerocopy")]
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
use musli_macros::Generate;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

use crate::generate::Generate;
pub use rand::prelude::*;

miri! {
    pub const PRIMITIVES_RANGE: Range<usize> = 10..100, 1..3;
    pub const MEDIUM_RANGE: Range<usize> = 10..100, 1..3;
}

#[derive(Debug, Clone, Copy, PartialEq, Generate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "musli-zerocopy", derive(ZeroCopy))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "zerocopy", derive(AsBytes, FromBytes, FromZeroes))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    archive(compare(PartialEq), check_bytes),
    archive_attr(derive(Debug))
)]
#[cfg_attr(any(feature = "musli-zerocopy", feature = "zerocopy"), repr(C))]
pub struct PrimitivesPacked {
    unsigned8: u8,
    _pad0: [u8; 1],
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    #[cfg(not(feature = "model-no-128"))]
    unsigned128: u128,
    signed8: i8,
    _pad1: [u8; 1],
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(not(feature = "model-no-128"))]
    signed128: i128,
    #[cfg(not(feature = "model-no-usize"))]
    unsignedsize: usize,
    #[cfg(not(feature = "model-no-usize"))]
    signedsize: isize,
    float32: f32,
    _pad3: [u8; 4],
    float64: f64,
}

#[cfg(feature = "rkyv")]
impl PartialEq<PrimitivesPacked> for &ArchivedPrimitivesPacked {
    #[inline]
    fn eq(&self, other: &PrimitivesPacked) -> bool {
        *other == **self
    }
}

impl PartialEq<PrimitivesPacked> for &PrimitivesPacked {
    #[inline]
    fn eq(&self, other: &PrimitivesPacked) -> bool {
        *other == **self
    }
}

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli-zerocopy", derive(ZeroCopy))]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
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
    #[cfg(not(feature = "model-no-128"))]
    unsigned128: u128,
    signed8: i8,
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(not(feature = "model-no-128"))]
    signed128: i128,
    #[cfg(not(feature = "model-no-usize"))]
    unsignedsize: usize,
    #[cfg(not(feature = "model-no-usize"))]
    signedsize: isize,
    #[cfg(not(feature = "model-no-float"))]
    float32: f32,
    #[cfg(not(feature = "model-no-float"))]
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
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
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
    #[cfg(not(feature = "model-no-cstring"))]
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
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
pub struct Tuples {
    u0: (),
    u1: (bool,),
    u2: (bool, u8),
    u3: (bool, u8, u32),
    u4: (bool, u8, u32, u64),
    #[cfg(not(feature = "model-no-float"))]
    u5: (bool, u8, u32, u64, f32),
    #[cfg(not(feature = "model-no-float"))]
    u6: (bool, u8, u32, u64, f32, f64),
    i0: (),
    i1: (bool,),
    i2: (bool, i8),
    i3: (bool, i8, i32),
    i4: (bool, i8, i32, i64),
    #[cfg(not(feature = "model-no-float"))]
    i5: (bool, i8, i32, i64, f32),
    #[cfg(not(feature = "model-no-float"))]
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
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
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
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
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
    #[cfg(not(any(feature = "model-no-vec", feature = "model-no-tuple")))]
    #[generate(range = PRIMITIVES_RANGE)]
    tuples: Vec<(Tuples, Tuples)>,
    #[generate(range = MEDIUM_RANGE)]
    medium_vec: Vec<MediumEnum>,
    #[cfg(not(feature = "model-no-map-string-key"))]
    #[generate(range = MEDIUM_RANGE)]
    medium_map: HashMap<String, MediumEnum>,
    #[cfg(not(feature = "model-no-map-string-key"))]
    string_keys: HashMap<String, u64>,
    #[cfg(not(feature = "model-no-map"))]
    number_keys: HashMap<u32, u64>,
    #[cfg(not(feature = "model-no-tuple"))]
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
