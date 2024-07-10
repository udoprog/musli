use crate::buf::Buf;
use crate::error::Error;

mod sealed {
    use crate::endian::ByteOrder;
    use crate::pointer::Size;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<K, V, E, O> Sealed for crate::phf::map::MapRef<K, V, E, O>
    where
        K: ZeroCopy,
        V: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
    }

    impl<K, V, E, O> Sealed for crate::swiss::map::MapRef<K, V, E, O>
    where
        K: ZeroCopy,
        V: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
    }

    impl<T, E, O> Sealed for crate::phf::set::SetRef<T, E, O>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
    }

    impl<T, E, O> Sealed for crate::swiss::set::SetRef<T, E, O>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
    }
}

/// Trait used for binding a reference to a [`Buf`] through [`Buf::bind()`].
///
/// This is used to make reference types easier to work with. Bound values
/// provide more natural APIs and can dereference to the underlying types.
pub trait Bindable: self::sealed::Sealed {
    /// The target of the binding.
    type Bound<'a>
    where
        Self: 'a;

    /// Bind the current value to a [`Buf`].
    fn bind(self, buf: &Buf) -> Result<Self::Bound<'_>, Error>;
}
