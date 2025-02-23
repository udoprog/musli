use crate::de::SizeHint;
use crate::Context;

/// A size hint passed in when encoding or decoding a map.
pub trait MapHint: Sized {
    /// Get the map hint.
    fn get(self) -> Option<usize>;

    /// Require the size of the map.
    #[inline]
    fn require<C>(self, cx: C) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let Some(size) = self.get() else {
            return Err(
                cx.message("Format cannot handle map types with an unknown number of entries")
            );
        };

        Ok(size)
    }

    /// The size hint for the map hint.
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
}

impl MapHint for Option<usize> {
    #[inline]
    fn get(self) -> Option<usize> {
        self
    }
}
