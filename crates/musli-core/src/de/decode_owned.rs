use crate::Allocator;

use super::Decode;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M, A>`.
pub trait DecodeOwned<M, A>: for<'de> Decode<'de, M, A>
where
    A: Allocator,
{
}

impl<M, D, A> DecodeOwned<M, A> for D
where
    D: for<'de> Decode<'de, M, A>,
    A: Allocator,
{
}
