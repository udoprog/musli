use crate::de::SizeHint;
use crate::Context;

/// A size hint passed in when encoding or decoding a sequence.
pub trait SequenceHint: Sized {
    /// Get the sequence hint.
    fn get(self) -> Option<usize>;

    /// Require the size of the sequence.
    #[inline]
    fn require<C>(self, cx: C) -> Result<usize, C::Error>
    where
        C: Context,
    {
        let Some(size) = self.get() else {
            return Err(
                cx.message("Format cannot handle sequence types with an unknown number of items")
            );
        };

        Ok(size)
    }

    /// The size hint for the sequence hint.
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
}

impl SequenceHint for Option<usize> {
    #[inline]
    fn get(self) -> Option<usize> {
        self
    }
}
