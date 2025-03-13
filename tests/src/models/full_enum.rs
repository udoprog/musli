#![cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]

#[cfg(not(feature = "no-nonunit-variant"))]
use alloc::string::String;
#[cfg(not(feature = "no-nonunit-variant"))]
use alloc::vec::Vec;

use crate::generate::Generate;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode), musli(mode = crate::mode::Packed))]
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
pub enum FullEnum {
    #[cfg(not(feature = "no-empty"))]
    Empty,
    #[cfg(not(feature = "no-nonunit-variant"))]
    EmptyTuple(),
    #[cfg_attr(feature = "musli", musli(transparent))]
    #[cfg(not(feature = "no-nonunit-variant"))]
    NewType(u64),
    #[cfg(not(feature = "no-nonunit-variant"))]
    Tuple(u64, u64),
    #[cfg_attr(feature = "musli", musli(transparent))]
    #[cfg(not(feature = "no-nonunit-variant"))]
    NewTypeString(String),
    #[cfg(not(feature = "no-nonunit-variant"))]
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
impl PartialEq<FullEnum> for &ArchivedMediumEnum {
    #[inline]
    fn eq(&self, other: &FullEnum) -> bool {
        *other == **self
    }
}

#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
impl PartialEq<FullEnum> for &FullEnum {
    #[inline]
    fn eq(&self, other: &FullEnum) -> bool {
        *other == **self
    }
}
