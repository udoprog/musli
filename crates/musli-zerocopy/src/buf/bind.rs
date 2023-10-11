use crate::buf::Buf;
use crate::error::Error;

mod sealed {
    use crate::pointer::Size;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<K: 'static, V: 'static, O: Size> Sealed for crate::map::MapRef<K, V, O>
    where
        K: ZeroCopy,
        V: ZeroCopy,
    {
    }
}

/// Trait used for binding a reference to a [`Buf`] through [`Buf::bind()`].
///
/// This is used to make reference types easier to work with. Bound values
/// provide more natural APIs and can dereference to the underlying types.
pub trait Bindable: self::sealed::Sealed {
    /// The target of the binding.
    type Bound<'a>;

    /// Bind the current value to a [`Buf`].
    fn bind(self, buf: &Buf) -> Result<Self::Bound<'_>, Error>;
}
