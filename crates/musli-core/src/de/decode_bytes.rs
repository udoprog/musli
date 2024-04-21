use super::Decoder;

/// Trait governing how types are decoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Decode` derive].
///
/// [`Decode` derive]: https://docs.rs/musli/latest/musli/help/derives/
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
/// use musli::{Decode, Decoder};
/// use musli::de::DecodeBytes;
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<'de, M> Decode<'de, M> for MyType {
///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de>,
///     {
///         Ok(Self {
///             data: DecodeBytes::decode_bytes(cx, decoder)?,
///         })
///     }
/// }
/// ```
pub trait DecodeBytes<'de, M>: Sized {
    /// Decode the given input as bytes.
    fn decode_bytes<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>;
}
