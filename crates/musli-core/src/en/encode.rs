use crate::en::Encoder;

/// Trait governing how types are encoded.
///
/// This is typically implemented automatically using the [`Encode` derive].
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
///     data: [u8; 128],
/// }
/// ```
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
///     type Encode = Self;
///
///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
///     where
///         E: Encoder<Mode = M>,
///     {
///         encoder.encode_array(&self.data)
///     }
///
///     #[inline]
///     fn as_encode(&self) -> &Self::Encode {
///         self
///     }
/// }
/// ```
pub trait Encode<M> {
    /// Whether the type is packed. Packed types can be bitwise copied if the
    /// representation of the serialization format is identical to the memory
    /// layout of the type.
    ///
    /// Note that setting this to `true` has safety implications, since it
    /// implies that assuming the type is correctly aligned it can be validly
    /// bitwise copied when encoded. Setting it to `false` is always safe.
    const ENCODE_PACKED: bool = false;

    /// The underlying type being encoded.
    ///
    /// This is used to "peek through" types like references being encoded.
    type Encode: ?Sized + Encode<M>;

    /// Encode the given output.
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>;

    /// Coerce into the underlying value being encoded.
    fn as_encode(&self) -> &Self::Encode;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
{
    const ENCODE_PACKED: bool = false;

    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(cx, encoder)
    }

    type Encode = T;

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
{
    const ENCODE_PACKED: bool = false;

    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(cx, encoder)
    }

    type Encode = T;

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}
