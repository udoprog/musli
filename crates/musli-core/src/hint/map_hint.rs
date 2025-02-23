use crate::de::SizeHint;
use crate::Context;

mod sealed {
    pub trait Sealed {}
    impl<T, M> Sealed for crate::__priv::EncodeMapHint<'_, T, M> where T: ?Sized {}
    impl Sealed for usize {}
    impl Sealed for Option<usize> {}
}

/// A size hint passed in when encoding or decoding a map.
pub trait MapHint: Sized + self::sealed::Sealed {
    /// Get an optional map hint.
    fn get(self) -> Option<usize>;

    /// Require a map hint or raise an error indicating that the format doesn't
    /// support maps of unknown size if one is not present.
    #[inline]
    fn require<C>(self, cx: C) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let Some(size) = self.get() else {
            return Err(cx.message("Format cannot handle maps with an unknown number of entries"));
        };

        Ok(size)
    }

    /// Coerce into a [`SizeHint`].
    #[inline]
    fn size_hint(self) -> SizeHint {
        match self.get() {
            Some(size) => SizeHint::exact(size),
            None => SizeHint::any(),
        }
    }
}

impl MapHint for usize {
    #[inline]
    fn get(self) -> Option<usize> {
        Some(self)
    }

    #[inline]
    fn require<C>(self, _: C) -> Result<usize, C::Error>
    where
        C: Context,
    {
        Ok(self)
    }

    #[inline]
    fn size_hint(self) -> SizeHint {
        SizeHint::exact(self)
    }
}

impl MapHint for Option<usize> {
    #[inline]
    fn get(self) -> Option<usize> {
        self
    }
}
