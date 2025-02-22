use crate::Allocator;

use super::Decoder;

/// Trait governing how types are decoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Decode` derive].
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
///     #[musli(bytes)]
///     data: [u8; 128],
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Allocator, Decode, Decoder};
/// use musli::de::DecodeBytes;
///
/// struct MyType {
///     data: [u8; 128],
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
///             data: DecodeBytes::decode_bytes(decoder)?,
///         })
///     }
/// }
/// ```
pub trait DecodeBytes<'de, M, A>: Sized
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
    const DECODE_BYTES_PACKED: bool = false;

    /// Decode the given input as bytes.
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>;
}
