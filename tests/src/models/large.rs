#[cfg(not(feature = "no-alloc-map"))]
use std::collections::HashMap;

#[cfg(not(all(
    feature = "no-tuple",
    any(feature = "no-alloc-map", feature = "no-str-key"),
)))]
use alloc::string::String;
use alloc::vec::Vec;

use crate::generate::Generate;

#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
use super::FullEnum;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode), musli(mode = crate::mode::Packed, packed))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "bincode-derive", derive(bincode::Encode, bincode::Decode))]
pub struct Large {
    #[generate(range = super::PRIMITIVES_RANGE.get())]
    primitives: Vec<super::Primitives>,
    #[cfg(all(not(feature = "no-alloc-vec"), not(feature = "no-tuple")))]
    #[generate(range = super::PRIMITIVES_RANGE.get())]
    tuples: Vec<(super::Tuples, super::Tuples)>,
    #[generate(range = super::MEDIUM_RANGE.get())]
    #[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
    medium_vec: Vec<FullEnum>,
    #[cfg(not(any(feature = "no-alloc-map", feature = "no-str-key")))]
    #[generate(range = super::MEDIUM_RANGE.get())]
    medium_map: HashMap<String, FullEnum>,
    #[cfg(not(feature = "no-tuple"))]
    string_vec: Vec<(String, u64)>,
    #[cfg(not(any(feature = "no-alloc-map", feature = "no-str-key")))]
    string_keys: HashMap<String, u64>,
    #[cfg(not(any(feature = "no-alloc-map", feature = "no-number-key")))]
    number_map: HashMap<u32, u64>,
    #[cfg(not(feature = "no-tuple"))]
    number_vec: Vec<(u32, u64)>,
}

crate::local_deref_sized! {
    Large,
    #[cfg(feature = "rkyv")]
    ArchivedLarge,
}
