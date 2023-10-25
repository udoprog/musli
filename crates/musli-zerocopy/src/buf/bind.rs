use crate::buf::Buf;
use crate::error::Error;

mod sealed {
    use crate::endian::ByteOrder;
    use crate::pointer::Size;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<K, V, O: Size, E: ByteOrder> Sealed for crate::phf::map::MapRef<K, V, O, E>
    where
        K: ZeroCopy,
        V: ZeroCopy,
    {
    }

    impl<K, V, O: Size, E: ByteOrder> Sealed for crate::swiss::map::MapRef<K, V, O, E>
    where
        K: ZeroCopy,
        V: ZeroCopy,
    {
    }

    impl<T, O: Size, E: ByteOrder> Sealed for crate::phf::set::SetRef<T, O, E> where T: ZeroCopy {}

    impl<T, O: Size, E: ByteOrder> Sealed for crate::swiss::set::SetRef<T, O, E> where T: ZeroCopy {}
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
