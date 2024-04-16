use crate::en::Encoder;
use crate::mode::DefaultMode;

/// Trait governing how a type is encoded as a packed value.
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
/// use musli::Encode;
///
/// #[derive(Encode)]
/// struct PackedType {
///     #[musli(packed)]
///     data: (u32, u32),
/// }
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Encode, Encoder};
/// use musli::en::PackEncoder;
///
/// struct PackedType {
///     data: (u32, u32),
/// }
///
/// impl<M> Encode<M> for PackedType {
///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
///     where
///         E: Encoder,
///     {
///         let mut pack = encoder.encode_pack()?;
///         pack.push(&self.data.0);
///         pack.push(&self.data.1);
///         pack.finish_pack()
///     }
/// }
/// ```
pub trait EncodePacked<M = DefaultMode> {
    /// Encode the given output as bytes.
    fn encode_packed<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>;
}

impl<T, M> EncodePacked<M> for &T
where
    T: ?Sized + EncodePacked<M>,
{
    #[inline]
    fn encode_packed<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_packed(cx, encoder)
    }
}

impl<T, M> EncodePacked<M> for &mut T
where
    T: ?Sized + EncodePacked<M>,
{
    #[inline]
    fn encode_packed<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_packed(cx, encoder)
    }
}
