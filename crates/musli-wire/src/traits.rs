use crate::types::{Kind, Tag};

/// Trait that encodes common behaviors of unsigned numbers.
pub trait Typed {
    /// The type flag used.
    const TYPE_FLAG: Tag;
}

macro_rules! implement {
    ($ty:ty, $type_flag:expr) => {
        impl Typed for $ty {
            const TYPE_FLAG: Tag = $type_flag;
        }
    };
}

implement!(u16, Tag::new(Kind::Fixed, 2));
implement!(u32, Tag::new(Kind::Fixed, 4));
implement!(u64, Tag::new(Kind::Fixed, 8));
implement!(u128, Tag::new(Kind::Fixed, 16));
// TODO: this needs to be easier to determine.
implement!(usize, Tag::new(Kind::Fixed, 8));
