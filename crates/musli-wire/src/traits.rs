use crate::types::TypeTag;

/// Trait that encodes common behaviors of unsigned numbers.
pub trait Typed {
    /// The type flag used.
    const TYPE_FLAG: TypeTag;
}

macro_rules! implement {
    ($ty:ty, $type_flag:ident) => {
        impl Typed for $ty {
            const TYPE_FLAG: TypeTag = TypeTag::$type_flag;
        }
    };
}

implement!(u16, Fixed16);
implement!(u32, Fixed32);
implement!(u64, Fixed64);
implement!(u128, Fixed128);
implement!(usize, Fixed32);
