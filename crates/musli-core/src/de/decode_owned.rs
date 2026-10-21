use crate::Allocator;
#[cfg(feature = "alloc")]
use crate::alloc::Global;

use super::Decode;

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M, A>`, and is
/// what you want when a function needs to decode a value which doesn't borrow
/// from the input, such as when it is decoded from a temporary buffer.
///
/// With the `alloc` feature enabled the allocator defaults to [`Global`], so
/// the bound can be written as just `DecodeOwned<Binary>`. Without it the
/// allocator must be specified.
///
/// # Examples
///
/// ```
/// use musli::DecodeOwned;
/// use musli::mode::Binary;
/// use musli::{Decode, Encode};
///
/// #[derive(Encode, Decode)]
/// struct Data {
///     value: u32,
/// }
///
/// fn roundtrip<T>(value: &T) -> T
/// where
///     T: Encode<Binary> + DecodeOwned<Binary>,
/// {
///     let bytes = musli::storage::to_vec(value).unwrap();
///     musli::storage::from_slice(&bytes[..]).unwrap()
/// }
///
/// assert_eq!(roundtrip(&Data { value: 42 }).value, 42);
/// ```
#[cfg(feature = "alloc")]
pub trait DecodeOwned<M, A = Global>: for<'de> Decode<'de, M, A>
where
    A: Allocator,
{
}

/// Decode to an owned value.
///
/// This is a simpler bound to use than `for<'de> Decode<'de, M, A>`.
///
/// With the `alloc` feature enabled the allocator defaults to `Global`.
#[cfg(not(feature = "alloc"))]
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
