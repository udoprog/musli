use crate::en::Encoder;
use crate::mode::DefaultMode;
use crate::Context;

/// Trait governing how a type is encoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Encode` derive].
///
/// [`Encode` derive]: https://docs.rs/musli/latest/musli/derives/
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
/// use musli::{Context, Encode, Encoder};
/// use musli::en::EncodeBytes;
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<M> Encode<M> for MyType {
///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
///     where
///         C: ?Sized + Context<Mode = M>,
///         E: Encoder<C>,
///     {
///         self.data.encode_bytes(cx, encoder)
///     }
/// }
/// ```
pub trait EncodeBytes<M = DefaultMode> {
    /// Encode the given output as bytes.
    fn encode_bytes<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>;
}

impl<T, M> EncodeBytes<M> for &T
where
    T: ?Sized + EncodeBytes<M>,
{
    #[inline]
    fn encode_bytes<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>,
    {
        T::encode_bytes(*self, cx, encoder)
    }
}

impl<T, M> EncodeBytes<M> for &mut T
where
    T: ?Sized + EncodeBytes<M>,
{
    #[inline]
    fn encode_bytes<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>,
    {
        T::encode_bytes(*self, cx, encoder)
    }
}
