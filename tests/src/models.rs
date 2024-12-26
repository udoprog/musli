#[cfg(all(feature = "std", not(feature = "no-map")))]
use std::collections::HashMap;
#[cfg(all(feature = "std", not(feature = "no-set"),))]
use std::collections::HashSet;

#[cfg(all(feature = "alloc", not(feature = "no-btree")))]
use alloc::collections::{BTreeMap, BTreeSet};

use core::ops::Range;

#[cfg(all(feature = "alloc", not(feature = "no-cstring")))]
use alloc::ffi::CString;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "musli-zerocopy")]
use musli_zerocopy::ZeroCopy;

#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, IntoBytes};

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

use crate::generate::Generate;
pub use rand::prelude::*;

miri! {
    pub const PRIMITIVES_RANGE: Range<usize> = 10..100, 1..3;
    pub const MEDIUM_RANGE: Range<usize> = 10..100, 1..3;
    pub const SMALL_FIELDS: Range<usize> = 1..3, 1..2;
}

#[derive(Debug, Clone, Copy, PartialEq, Generate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "musli", derive(Encode, Decode))]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
#[cfg_attr(feature = "musli-zerocopy", derive(ZeroCopy))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "zerocopy", derive(IntoBytes, FromBytes, Immutable))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(any(feature = "musli-zerocopy", feature = "zerocopy"), repr(C))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct PrimitivesPacked {
    unsigned8: u8,
    #[cfg_attr(feature = "musli", musli(bytes))]
    _pad0: [u8; 1],
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    #[cfg(not(feature = "no-128"))]
    unsigned128: u128,
    signed8: i8,
    #[cfg_attr(feature = "musli", musli(bytes))]
    _pad1: [u8; 1],
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(not(feature = "no-128"))]
    signed128: i128,
    #[cfg(not(feature = "no-usize"))]
    unsignedsize: usize,
    #[cfg(not(feature = "no-isize"))]
    signedsize: isize,
    float32: f32,
    #[cfg_attr(feature = "musli", musli(bytes))]
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
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
#[cfg_attr(feature = "musli-zerocopy", repr(C))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Primitives {
    boolean: bool,
    #[cfg(not(feature = "no-char"))]
    character: char,
    unsigned8: u8,
    unsigned16: u16,
    unsigned32: u32,
    unsigned64: u64,
    #[cfg(not(feature = "no-128"))]
    unsigned128: u128,
    signed8: i8,
    signed16: i16,
    signed32: i32,
    signed64: i64,
    #[cfg(not(feature = "no-128"))]
    signed128: i128,
    #[cfg(not(feature = "no-usize"))]
    unsignedsize: usize,
    #[cfg(not(feature = "no-isize"))]
    signedsize: isize,
    #[cfg(not(feature = "no-float"))]
    float32: f32,
    #[cfg(not(feature = "no-float"))]
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
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Allocated {
    #[cfg(feature = "alloc")]
    string: String,
    #[cfg_attr(feature = "musli", musli(bytes))]
    #[cfg(feature = "alloc")]
    #[generate(range = SMALL_FIELDS)]
    bytes: Vec<u8>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-number-key")
    ))]
    #[generate(range = SMALL_FIELDS)]
    number_map: HashMap<u32, u64>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-string-key")
    ))]
    #[generate(range = SMALL_FIELDS)]
    string_map: HashMap<String, u64>,
    #[generate(range = SMALL_FIELDS)]
    #[cfg(all(feature = "std", not(feature = "no-set"),))]
    number_set: HashSet<u32>,
    #[generate(range = SMALL_FIELDS)]
    #[cfg(all(
        feature = "std",
        not(feature = "no-set"),
        not(feature = "no-string-set")
    ))]
    string_set: HashSet<String>,
    #[cfg(all(
        feature = "alloc",
        not(feature = "no-btree"),
        not(feature = "no-number-key")
    ))]
    #[generate(range = SMALL_FIELDS)]
    number_btree: BTreeMap<u32, u64>,
    #[cfg(all(feature = "alloc", not(feature = "no-btree")))]
    #[generate(range = SMALL_FIELDS)]
    string_btree: BTreeMap<String, u64>,
    #[cfg(all(feature = "alloc", not(feature = "no-btree")))]
    #[generate(range = SMALL_FIELDS)]
    number_btree_set: BTreeSet<u32>,
    #[cfg(all(feature = "alloc", not(feature = "no-btree")))]
    #[generate(range = SMALL_FIELDS)]
    string_btree_set: BTreeSet<String>,
    #[cfg(all(feature = "alloc", not(feature = "no-cstring")))]
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
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Tuples {
    u0: (),
    u1: (bool,),
    u2: (bool, u8),
    u3: (bool, u8, u32),
    u4: (bool, u8, u32, u64),
    #[cfg(not(feature = "no-float"))]
    u5: (bool, u8, u32, u64, f32),
    #[cfg(not(feature = "no-float"))]
    u6: (bool, u8, u32, u64, f32, f64),
    i0: (),
    i1: (bool,),
    i2: (bool, i8),
    i3: (bool, i8, i32),
    i4: (bool, i8, i32, i64),
    #[cfg(not(feature = "no-float"))]
    i5: (bool, i8, i32, i64, f32),
    #[cfg(not(feature = "no-float"))]
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
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum MediumEnum {
    #[cfg(not(feature = "no-empty"))]
    Empty,
    #[cfg(not(feature = "no-nonunit-variant"))]
    EmptyTuple(),
    #[cfg_attr(feature = "musli", musli(transparent))]
    #[cfg(all(not(feature = "no-newtype"), not(feature = "no-nonunit-variant")))]
    NewType(u64),
    #[cfg(not(feature = "no-nonunit-variant"))]
    Tuple(u64, u64),
    #[cfg_attr(feature = "musli", musli(transparent))]
    #[cfg(all(
        feature = "alloc",
        not(feature = "no-newtype"),
        not(feature = "no-nonunit-variant")
    ))]
    NewTypeString(String),
    #[cfg(all(feature = "alloc", not(feature = "no-nonunit-variant")))]
    TupleString(String, Vec<u8>),
    #[cfg(not(feature = "no-nonunit-variant"))]
    Struct {
        a: u32,
        primitives: Primitives,
        b: u64,
    },
    #[cfg(not(feature = "no-nonunit-variant"))]
    EmptyStruct {},
}

#[cfg(all(
    feature = "rkyv",
    any(not(feature = "no-empty"), not(feature = "no-nonunit-variant"))
))]
impl PartialEq<MediumEnum> for &ArchivedMediumEnum {
    #[inline]
    fn eq(&self, other: &MediumEnum) -> bool {
        *other == **self
    }
}

#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
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
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(feature = "musli", musli(mode = Packed, packed))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct LargeStruct {
    #[generate(range = PRIMITIVES_RANGE)]
    #[cfg(feature = "alloc")]
    primitives: Vec<Primitives>,
    #[cfg(all(feature = "alloc", not(feature = "no-vec"), not(feature = "no-tuple")))]
    #[generate(range = PRIMITIVES_RANGE)]
    tuples: Vec<(Tuples, Tuples)>,
    #[generate(range = MEDIUM_RANGE)]
    #[cfg(all(
        feature = "alloc",
        any(not(feature = "no-empty"), not(feature = "no-nonunit-variant"))
    ))]
    medium_vec: Vec<MediumEnum>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-string-key")
    ))]
    #[generate(range = MEDIUM_RANGE)]
    medium_map: HashMap<String, MediumEnum>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-string-key")
    ))]
    string_keys: HashMap<String, u64>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-number-key")
    ))]
    number_map: HashMap<u32, u64>,
    #[cfg(all(feature = "alloc", not(feature = "no-tuple")))]
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
