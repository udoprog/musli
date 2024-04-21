use crate::en::Encoder;

/// Trait governing how a type is encoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Encode` derive].
///
/// [`Encode` derive]: https://docs.rs/musli/latest/musli/help/derives/
///
/// # Examples
///
/// ```
/// use musli::Encode;
///
/// #[derive(Encode)]
/// struct MyType {
///     #[musli(bytes)]
///     data: [u8; 128],
/// }
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Encode, Encoder};
/// use musli::en::EncodeBytes;
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<M> Encode<M> for MyType {
///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
///     where
///         E: Encoder,
///     {
///         self.data.encode_bytes(cx, encoder)
///     }
/// }
/// ```
pub trait EncodeBytes<M> {
    /// Encode the given output as bytes.
    fn encode_bytes<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>;
}

impl<T, M> EncodeBytes<M> for &T
where
    T: ?Sized + EncodeBytes<M>,
{
    #[inline]
    fn encode_bytes<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_bytes(cx, encoder)
    }
}

impl<T, M> EncodeBytes<M> for &mut T
where
    T: ?Sized + EncodeBytes<M>,
{
    #[inline]
    fn encode_bytes<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_bytes(cx, encoder)
    }
}
