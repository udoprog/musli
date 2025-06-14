use crate::generate::Generate;

#[derive(Debug, Clone, PartialEq, Generate)]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode), musli(mode = crate::mode::Packed, packed))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
#[cfg_attr(feature = "bincode-derive", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "facet", derive(facet::Facet))]
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
