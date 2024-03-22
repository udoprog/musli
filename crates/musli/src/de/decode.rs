use crate::de::Decoder;
use crate::mode::DefaultMode;
use crate::Context;

/// Please refer to the main [musli documentation](https://docs.rs/musli).
#[doc(inline)]
pub use musli_macros::Decode;

/// Trait governing how types are decoded.
///
/// This is typically implemented automatically using the [`Decode` derive].
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
///     data: [u8; 128],
/// }
/// ````
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
///             data: decoder.decode_array(cx)?,
///         })
///     }
/// }
/// ```
pub trait Decode<'de, M = DefaultMode>: Sized {
    /// Decode the given input.
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, <D::Cx as Context>::Error>
    where
        D: Decoder<'de, Mode = M>;
}

/// Trait governing how types are decoded specifically for tracing.
///
/// This is used for types where some extra bounds might be necessary to trace a
/// container such as a [`HashMap<K, V>`] where `K` would have to implement
/// [`fmt::Display`].
///
/// [`HashMap<K, V>`]: std::collections::HashMap
/// [`fmt::Display`]: std::fmt::Display
pub trait TraceDecode<'de, M = DefaultMode>: Sized {
    /// Decode the given input.
    fn trace_decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>;
}
