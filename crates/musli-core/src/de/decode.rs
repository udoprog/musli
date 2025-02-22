use crate::Allocator;

use super::Decoder;

/// Trait governing how types are decoded.
///
/// This is typically implemented automatically using the [`Decode` derive].
///
/// [`Decode` derive]: https://docs.rs/musli/latest/musli/_help/derives/
///
/// # Examples
///
/// ```
/// use musli::Decode;
///
/// #[derive(Decode)]
/// struct MyType {
///     data: [u32; 8],
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Allocator, Decode, Decoder};
///
/// struct MyType {
///     data: [u32; 8],
/// }
///
/// impl<'de, M, A> Decode<'de, M, A> for MyType
/// where
///     A: Allocator,
/// {
///     #[inline]
///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de>,
///     {
///         Ok(Self {
///             data: decoder.decode()?,
///         })
///     }
/// }
/// ```
pub trait Decode<'de, M, A>: Sized
where
    A: Allocator,
{
    /// Whether the type is packed. Packed types can be bitwise copied if the
    /// representation of the serialization format is identical to the memory
    /// layout of the type.
    ///
    /// Note that setting this to `true` has safety implications, since it
    /// implies that assuming the type is correctly aligned it can be validly
    /// bitwise copied when encoded. Setting it to `false` is always safe.
    ///
    /// This being set to `true` also implies that the type is `Copy`, and must
    /// not have a `Drop` implementation.
    const IS_BITWISE_DECODE: bool = false;

    /// Decode the current value.
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>;
}
