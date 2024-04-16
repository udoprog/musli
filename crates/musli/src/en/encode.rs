use crate::en::Encoder;

pub use musli_macros::Encode;

/// Trait governing how types are encoded.
///
/// This is typically implemented automatically using the [`Encode` derive].
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
///     data: [u8; 128],
/// }
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Encode, Encoder};
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<M> Encode<M> for MyType {
///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
///     where
///         E: Encoder<Mode = M>,
///     {
///         encoder.encode_array(&self.data)
///     }
/// }
/// ```
pub trait Encode<M> {
    /// Encode the given output.
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(cx, encoder)
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(cx, encoder)
    }
}
