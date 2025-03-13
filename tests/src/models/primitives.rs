#[cfg(feature = "musli-zerocopy")]
use musli_zerocopy::ZeroCopy;

#[cfg(feature = "epserde")]
use epserde::Epserde;

#[cfg(feature = "musli")]
use musli::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "musli")]
use crate::mode::Packed;

use crate::generate::Generate;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "musli", derive(Encode, Decode), musli(mode = Packed, packed))]
#[cfg_attr(feature = "musli-zerocopy", derive(ZeroCopy))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(any(feature = "musli-zerocopy", feature = "epserde"), repr(C))]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
#[cfg_attr(feature = "epserde", derive(Copy, Epserde), zero_copy)]
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
