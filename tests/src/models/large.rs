#[cfg(all(feature = "std", not(feature = "no-map")))]
use std::collections::HashMap;

#[cfg(all(
    feature = "std",
    not(feature = "no-map"),
    not(feature = "no-string-key")
))]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

use crate::generate::Generate;

#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
use super::MediumEnum;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "musli", derive(Encode, Decode), musli(mode = Packed, packed))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct LargeStruct {
    #[generate(range = super::PRIMITIVES_RANGE.get())]
    #[cfg(feature = "alloc")]
    primitives: Vec<super::Primitives>,
    #[cfg(all(feature = "alloc", not(feature = "no-vec"), not(feature = "no-tuple")))]
    #[generate(range = super::PRIMITIVES_RANGE.get())]
    tuples: Vec<(super::Tuples, super::Tuples)>,
    #[generate(range = super::MEDIUM_RANGE.get())]
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
    #[generate(range = super::MEDIUM_RANGE.get())]
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
