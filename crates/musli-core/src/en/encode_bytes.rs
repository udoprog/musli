use crate::en::Encoder;

/// Trait governing how a type is encoded as bytes.
///
/// This is typically used automatically through the `#[musli(bytes)]` attribute
/// through the [`Encode` derive].
///
/// [`Encode` derive]: https://docs.rs/musli/latest/musli/_help/derives/
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
/// ```
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
///     type Encode = Self;
///
///     #[inline]
///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
///     where
///         E: Encoder,
///     {
///         self.data.encode_bytes(encoder)
///     }
///
///     #[inline]
///     fn as_encode(&self) -> &Self::Encode {
///         self
///     }
/// }
/// ```
pub trait EncodeBytes<M> {
    /// Whether the type is packed. Packed types can be bitwise copied if the
    /// representation of the serialization format is identical to the memory
    /// layout of the type.
    ///
    /// Note that setting this to `true` has safety implications, since it
    /// implies that assuming the type is correctly aligned it can be validly
    /// bitwise copied when encoded. Setting it to `false` is always safe.
    const ENCODE_BYTES_PACKED: bool = false;

    /// The underlying type being encoded.
    ///
    /// This is used to "peek through" types like references being encoded.
    type EncodeBytes: ?Sized + EncodeBytes<M>;

    /// Encode the given output as bytes.
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>;

    /// Coerce into the underlying value being encoded.
    fn as_encode_bytes(&self) -> &Self::EncodeBytes;
}

impl<T, M> EncodeBytes<M> for &T
where
    T: ?Sized + EncodeBytes<M>,
{
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = T;

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_bytes(encoder)
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<T, M> EncodeBytes<M> for &mut T
where
    T: ?Sized + EncodeBytes<M>,
{
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = T;

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode_bytes(encoder)
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}
