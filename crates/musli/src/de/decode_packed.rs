use crate::de::Decoder;
use crate::mode::DefaultMode;
use crate::Context;

/// Trait governing how a type is decoded as a packed value.
///
/// Packed encodings are ones where data follow one after another, with no
/// "metadata" indicating when one value starts and another stops.
///
/// This is typically used automatically through the `#[musli(packed)]`
/// attribute through the [`Decode` derive].
///
/// [`Decode` derive]: https://docs.rs/musli/latest/musli/derives/
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
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Decode, Decoder};
/// use musli::de::PackDecoder;
///
/// struct Packed {
///     data: (u32, u32),
/// }
///
/// impl<'de, M> Decode<'de, M> for Packed {
///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
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
pub trait DecodePacked<'de, M = DefaultMode>: Sized {
    /// Decode the given input as bytes.
    fn decode_packed<D>(cx: &D::Cx, decoder: D) -> Result<Self, <D::Cx as Context>::Error>
    where
        D: Decoder<'de, Mode = M>;
}
