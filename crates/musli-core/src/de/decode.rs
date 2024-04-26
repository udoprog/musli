use super::Decoder;

/// Trait governing how types are decoded.
///
/// This is typically implemented automatically using the [`Decode` derive].
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
///     data: [u8; 128],
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Decode, Decoder};
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
///             data: decoder.decode_array()?,
///         })
///     }
/// }
/// ```
pub trait Decode<'de, M>: Sized {
    /// Decode the given input.
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>;
}
