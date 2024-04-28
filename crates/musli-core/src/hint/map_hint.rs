use crate::de::SizeHint;

/// A hint passed in when encoding a map.
#[non_exhaustive]
pub struct MapHint {
    /// The size for the map being encoded.
    pub size: usize,
}

impl MapHint {
    /// Create a new struct hint with the specified size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::hint::MapHint;
    ///
    /// static HINT: MapHint = MapHint::with_size(16);
    ///
    /// assert_eq!(HINT.size, 16);
    /// ```
    #[inline]
    pub const fn with_size(size: usize) -> Self {
        Self { size }
    }

    /// Return the size hint that corresponds to this overall hint.
    #[inline]
    pub fn size_hint(&self) -> SizeHint {
        SizeHint::exact(self.size)
    }
}
