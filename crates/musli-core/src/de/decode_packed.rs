use crate::Allocator;

use super::Decoder;

/// Trait governing how a type is decoded as a packed value.
///
/// Packed encodings are ones where data follow one after another, with no
/// "metadata" indicating when one value starts and another stops.
///
/// This is typically used automatically through the `#[musli(packed)]`
/// attribute through the [`Decode` derive].
///
/// [`Decode` derive]: https://docs.rs/musli/latest/musli/_help/derives/
///
/// # Examples
///
/// ```
/// use musli::Decode;
///
/// #[derive(Decode)]
/// struct Packed {
///     #[musli(packed)]
///     data: (u32, u32),
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Allocator, Decode, Decoder};
/// use musli::de::SequenceDecoder;
///
/// struct Packed {
///     data: (u32, u32),
/// }
///
/// impl<'de, M, A> Decode<'de, M, A> for Packed
/// where
///     A: Allocator,
/// {
///     #[inline]
///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de>,
///     {
///         decoder.decode_pack(|pack| {
///             Ok(Self {
///                 data: (pack.next()?, pack.next()?),
///             })
///         })
///     }
/// }
/// ```
pub trait DecodePacked<'de, M, A>: Sized
where
    A: Allocator,
{
    /// Decode the given input as bytes.
    fn decode_packed<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>;
}
