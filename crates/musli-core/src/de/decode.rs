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
///     data: [u8; 128],
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Allocator, Decode, Decoder};
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<'de, M, A> Decode<'de, M, A> for MyType
/// where
///     A: Allocator,
/// {
///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de>,
///     {
///         Ok(Self {
///             data: decoder.decode_array()?,
///         })
///     }
/// }
/// ```
pub trait Decode<'de, M, A>
where
    Self: Sized,
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

    /// Decode the given input.
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>;
}
