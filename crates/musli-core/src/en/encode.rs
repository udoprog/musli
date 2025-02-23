use crate::en::Encoder;

/// Trait governing how types are encoded.
///
/// This is typically implemented automatically using the [`Encode` derive].
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
///     data: [u32; 8],
/// }
/// ```
///
/// Implementing manually:
///
/// ```
/// use musli::{Encode, Encoder};
///
/// struct MyType {
///     data: [u32; 8],
/// }
///
/// impl<M> Encode<M> for MyType {
///     type Encode = Self;
///
///     #[inline]
///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
///     where
///         E: Encoder<Mode = M>,
///     {
///         encoder.encode(&self.data)
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
    ///
    /// This being set to `true` also implies that the type is `Copy`, and must
    /// not have a `Drop` implementation.
    const IS_BITWISE_ENCODE: bool = false;

    /// The underlying type being encoded.
    ///
    /// This is used to "peek through" types like references being encoded.
    type Encode: ?Sized + Encode<M>;

    /// Encode the given output.
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>;

    /// The number of fields in the type.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Coerce into the underlying value being encoded.
    fn as_encode(&self) -> &Self::Encode;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
{
    type Encode = T;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(encoder)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        (**self).size_hint()
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
{
    type Encode = T;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        (**self).encode(encoder)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        (**self).size_hint()
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}
