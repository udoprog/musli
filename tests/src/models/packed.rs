use crate::generate::Generate;

#[derive(Debug, Clone, Copy, PartialEq, Generate)]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode), musli(mode = crate::mode::Packed, packed))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli-zerocopy", derive(musli_zerocopy::ZeroCopy))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::IntoBytes, zerocopy::FromBytes, zerocopy::Immutable)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    rkyv(compare(PartialEq), derive(Debug))
)]
#[cfg_attr(
    any(feature = "musli-zerocopy", feature = "zerocopy", feature = "epserde"),
    repr(C)
)]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
#[cfg_attr(feature = "epserde", derive(epserde::Epserde), zero_copy)]
#[cfg_attr(feature = "bincode-derive", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "facet", derive(facet::Facet))]
pub struct Packed {
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
impl PartialEq<Packed> for &ArchivedPacked {
    #[inline]
    fn eq(&self, other: &Packed) -> bool {
        *other == **self
    }
}

impl PartialEq<Packed> for &Packed {
    #[inline]
    fn eq(&self, other: &Packed) -> bool {
        *other == **self
    }
}
