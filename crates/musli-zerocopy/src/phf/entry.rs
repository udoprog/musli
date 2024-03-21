use crate::ZeroCopy;

/// An entry which is used when constructing a [`Map<K, V>`].
///
/// To construct a map, this type is used to provide [`OwnedBuf`] with a pair of
/// values.
///
/// Note that this primarily exists because tuples are not support. The layout
/// of a tuple is `repr(Rust)`, so there is no way to construct legal references
/// to them.
///
/// [`Map<K, V>`]: crate::phf::Map
/// [`OwnedBuf`]: crate::buf::OwnedBuf
#[derive(Debug, ZeroCopy)]
#[zero_copy(crate, bounds = {K: ZeroCopy, V: ZeroCopy})]
#[repr(C)]
pub(crate) struct Entry<K, V> {
    /// The first element in the pair.
    pub(crate) key: K,
    /// The second element in the pair.
    pub(crate) value: V,
}

impl<K, V> Entry<K, V> {
    /// Construct a new pair.
    #[cfg(feature = "alloc")]
    pub(crate) const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}
