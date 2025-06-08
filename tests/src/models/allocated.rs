#[cfg(all(feature = "std", not(feature = "no-map")))]
use std::collections::HashMap;
#[cfg(all(feature = "std", not(feature = "no-set")))]
use std::collections::HashSet;

#[cfg(not(feature = "no-btree"))]
use alloc::collections::{BTreeMap, BTreeSet};

#[cfg(not(feature = "no-cstring"))]
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

use crate::generate::Generate;

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
#[cfg_attr(feature = "facet", derive(facet::Facet))]
pub struct Allocated {
    #[cfg(feature = "alloc")]
    string: String,
    #[cfg_attr(feature = "musli", musli(bytes))]
    #[cfg(feature = "alloc")]
    #[generate(range = super::SMALL_FIELDS.get())]
    bytes: Vec<u8>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-number-key")
    ))]
    #[generate(range = super::SMALL_FIELDS.get())]
    number_map: HashMap<u32, u64>,
    #[cfg(all(
        feature = "std",
        not(feature = "no-map"),
        not(feature = "no-string-key")
    ))]
    #[generate(range = super::SMALL_FIELDS.get())]
    string_map: HashMap<String, u64>,
    #[generate(range = super::SMALL_FIELDS.get())]
    #[cfg(all(feature = "std", not(feature = "no-set")))]
    number_set: HashSet<u32>,
    #[generate(range = super::SMALL_FIELDS.get())]
    #[cfg(all(
        feature = "std",
        not(feature = "no-set"),
        not(feature = "no-string-set")
    ))]
    string_set: HashSet<String>,
    #[cfg(all(not(feature = "no-btree"), not(feature = "no-number-key")))]
    #[generate(range = super::SMALL_FIELDS.get())]
    number_btree: BTreeMap<u32, u64>,
    #[cfg(not(feature = "no-btree"))]
    #[generate(range = super::SMALL_FIELDS.get())]
    string_btree: BTreeMap<String, u64>,
    #[cfg(not(feature = "no-btree"))]
    #[generate(range = super::SMALL_FIELDS.get())]
    number_btree_set: BTreeSet<u32>,
    #[cfg(not(feature = "no-btree"))]
    #[generate(range = super::SMALL_FIELDS.get())]
    string_btree_set: BTreeSet<String>,
    #[cfg(not(feature = "no-cstring"))]
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
