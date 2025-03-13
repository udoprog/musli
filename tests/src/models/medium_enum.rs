#![cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]

#[cfg(all(feature = "alloc", not(feature = "no-nonunit-variant")))]
use alloc::string::String;
#[cfg(all(feature = "alloc", not(feature = "no-nonunit-variant")))]
use alloc::vec::Vec;

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

use crate::generate::Generate;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "musli", derive(Encode, Decode), musli(mode = Packed))]
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
        primitives: super::Primitives,
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
