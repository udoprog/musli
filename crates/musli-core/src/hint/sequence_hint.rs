use crate::de::SizeHint;
use crate::Context;

mod sealed {
    pub trait Sealed {}
    impl Sealed for usize {}
    impl Sealed for Option<usize> {}
}

/// A size hint passed in when encoding or decoding a sequence.
pub trait SequenceHint: Sized + self::sealed::Sealed {
    /// Get an optional sequence hint.
    fn get(self) -> Option<usize>;

    /// Require a sequence hint or raise an error indicating that the format
    /// doesn't support sequences of unknown size if one is not present.
    #[inline]
    fn require<C>(self, cx: C) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let Some(size) = self.get() else {
            return Err(
                cx.message("Format cannot handle sequences with an unknown number of items")
            );
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

impl SequenceHint for usize {
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

impl SequenceHint for Option<usize> {
    #[inline]
    fn get(self) -> Option<usize> {
        self
    }
}
