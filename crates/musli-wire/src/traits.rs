use crate::types::{FIXED128, FIXED16, FIXED32, FIXED64};

/// Trait that encodes common behaviors of unsigned numbers.
pub trait Typed {
    /// The type flag used.
    const TYPE_FLAG: u8;
}

macro_rules! implement {
    ($ty:ty, $type_flag:expr) => {
        impl Typed for $ty {
            const TYPE_FLAG: u8 = $type_flag;
        }
    };
}

implement!(u16, FIXED16);
implement!(u32, FIXED32);
implement!(u64, FIXED64);
implement!(u128, FIXED128);
implement!(usize, FIXED32);
