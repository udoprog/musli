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
#[cfg_attr(feature = "bitcode-derive", derive(bitcode::Encode, bitcode::Decode))]
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
