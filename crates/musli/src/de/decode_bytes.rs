use crate::de::Decoder;
use crate::mode::DefaultMode;
use crate::Context;

/// Trait governing how types are decoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Decode` derive].
///
/// [`Decode` derive]: https://docs.rs/musli/latest/musli/derives/
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
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Context, Decode, Decoder};
/// use musli::de::DecodeBytes;
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<'de, M> Decode<'de, M> for MyType {
///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
///     where
///         C: ?Sized + Context<Mode = M>,
///         D: Decoder<'de, C>,
///     {
///         Ok(Self {
///             data: DecodeBytes::decode_bytes(cx, decoder)?,
///         })
///     }
/// }
/// ```
pub trait DecodeBytes<'de, M = DefaultMode>: Sized {
    /// Decode the given input as bytes.
    fn decode_bytes<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        D: Decoder<'de, C>;
}
