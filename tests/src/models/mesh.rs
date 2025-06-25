use alloc::vec::Vec;

use crate::generate::Generate;

#[derive(Debug, Clone, Generate)]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode), musli(mode = crate::mode::Packed, packed))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize),
    rkyv(compare(PartialEq), derive(Debug), archive_bounds(V::Archived: core::fmt::Debug))
)]
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
#[cfg_attr(feature = "epserde", derive(epserde::Epserde))]
pub struct Mesh<V: AsRef<[Triangle]> = Vec<Triangle>> {
    pub triangles: V,
}

impl<A, B> PartialEq<Mesh<A>> for Mesh<B>
where
    A: AsRef<[Triangle]>,
    B: AsRef<[Triangle]>,
{
    #[inline]
    fn eq(&self, other: &Mesh<A>) -> bool {
        self.triangles.as_ref() == other.triangles.as_ref()
    }
}

crate::local_deref_sized! {
    {T} Mesh<T> where T: AsRef<[Triangle]>,
    #[cfg(feature = "rkyv")] ArchivedMesh,
}

#[derive(Debug, Clone, Copy, PartialEq, Generate)]
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
#[cfg_attr(feature = "epserde", derive(epserde::Epserde), repr(C), zero_copy)]
#[cfg_attr(feature = "bincode-derive", derive(bincode::Encode, bincode::Decode))]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Generate)]
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
#[cfg_attr(feature = "epserde", derive(epserde::Epserde), repr(C), zero_copy)]
#[cfg_attr(feature = "bincode-derive", derive(bincode::Encode, bincode::Decode))]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
