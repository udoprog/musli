//! A map which implements a hash-map like interface, where values can be looked
//! up by keys.
//!
//! This map are implemented using a perfect hash functions, and are inserted
//! into a buffering using [`AlignedBuf::insert_map`].
//!
//! There's two types provided by this module:
//! * [`Map<K, V>`] which is a *bound* reference to a map, providing a
//!   convenient map-like access.
//! * [`MapRef<K, V>`] which is the *pointer* of the map. This is what you store
//!   in [`ZeroCopy`] types and is what is returned by
//!   [`AlignedBuf::insert_map`].
//!
//! [`AlignedBuf::insert_map`]: crate::buf::AlignedBuf::insert_map

pub use self::entry::Entry;
mod entry;

use core::borrow::Borrow;
use core::hash::Hash;

use crate::buf::{Bindable, Buf, Visit};
use crate::error::Error;
use crate::phf::hashing::HashKey;
use crate::pointer::{DefaultSize, Size, Slice};
use crate::traits::ZeroCopy;

/// A map bound to a [`Buf`] through [`Buf::bind`] for convenience.
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::map::Entry;
///
/// let mut buf = AlignedBuf::new();
///
/// let mut map = Vec::new();
///
/// map.push(Entry::new(1, 2));
/// map.push(Entry::new(2, 3));
///
/// let map = buf.insert_map(&mut map)?;
/// let buf = buf.as_aligned();
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&1)?, Some(&2));
/// assert_eq!(map.get(&2)?, Some(&3));
/// assert_eq!(map.get(&3)?, None);
///
/// assert!(map.contains_key(&1)?);
/// assert!(!map.contains_key(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub struct Map<'a, K, V> {
    key: HashKey,
    entries: &'a [Entry<K, V>],
    displacements: &'a [Entry<u32, u32>],
    buf: &'a Buf,
}

impl<'a, K, V> Map<'a, K, V>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    /// Get a value from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get(&1)?, Some(&2));
    /// assert_eq!(map.get(&2)?, Some(&3));
    /// assert_eq!(map.get(&3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<T>(&self, key: &T) -> Result<Option<&V>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Visit,
        K::Target: Borrow<T>,
    {
        let Some(entry) = self.get_entry(key)? else {
            return Ok(None);
        };

        Ok(Some(entry.1))
    }

    /// Test if the map contains the given `key`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert!(map.contains_key(&1)?);
    /// assert!(map.contains_key(&2)?);
    /// assert!(!map.contains_key(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains_key<T>(&self, key: &T) -> Result<bool, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Visit,
        K::Target: Borrow<T>,
    {
        Ok(self.get_entry(key)?.is_some())
    }

    /// Get an entry from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get_entry(&1)?, Some((&1, &2)));
    /// assert_eq!(map.get_entry(&2)?, Some((&2, &3)));
    /// assert_eq!(map.get_entry(&3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get_entry<T>(&self, key: &T) -> Result<Option<(&K, &V)>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Visit,
        K::Target: Borrow<T>,
    {
        if self.displacements.is_empty() {
            return Ok(None);
        }

        let hashes = crate::phf::hashing::hash(key, &self.key);
        let index =
            crate::phf::hashing::get_index(&hashes, self.displacements, self.entries.len())?;

        let Some(e) = self.entries.get(index) else {
            return Ok(None);
        };

        if e.key.visit(self.buf, |v| v.borrow() == key)? {
            Ok(Some((&e.key, &e.value)))
        } else {
            Ok(None)
        }
    }
}

/// Bind a [`MapRef`] into a [`Map`].
impl<K, V, O: Size> Bindable for MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    type Bound<'a> = Map<'a, K, V> where Self: 'a;

    fn bind(self, buf: &Buf) -> Result<Self::Bound<'_>, Error> {
        Ok(Map {
            key: self.key,
            entries: buf.load(self.entries)?,
            displacements: buf.load(self.displacements)?,
            buf,
        })
    }
}

/// A stored reference to a map.
///
/// Note that operating over the methods provided in [`MapRef`] does not demand
/// that the entire contents of the set is validated as would be the case when
/// [`bind()`] is used and might result in better performance if the data is
/// infrequently accessed.
///
/// Constructed through [`AlignedBuf::insert_map`].
///
/// [`AlignedBuf::insert_map`]: crate::buf::AlignedBuf::insert_map
/// [`bind()`]: crate::buf::Buf::bind
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::map::Entry;
///
/// let mut buf = AlignedBuf::new();
///
/// let mut map = Vec::new();
///
/// map.push(Entry::new(1, 2));
/// map.push(Entry::new(2, 3));
///
/// let map = buf.insert_map(&mut map)?;
/// let buf = buf.as_aligned();
///
/// assert_eq!(map.get(buf, &1)?, Some(&2));
/// assert_eq!(map.get(buf, &2)?, Some(&3));
/// assert_eq!(map.get(buf, &3)?, None);
///
/// assert!(map.contains_key(buf, &1)?);
/// assert!(!map.contains_key(buf, &3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug)]
pub struct MapRef<K, V, O: Size = DefaultSize>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    key: HashKey,
    entries: Slice<Entry<K, V>, O>,
    displacements: Slice<Entry<u32, u32>, O>,
}

impl<K, V, O: Size> MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    #[cfg(feature = "alloc")]
    pub(crate) fn new(
        key: HashKey,
        entries: Slice<Entry<K, V>, O>,
        displacements: Slice<Entry<u32, u32>, O>,
    ) -> Self {
        Self {
            key,
            entries,
            displacements,
        }
    }
}

impl<K, V, O: Size> MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    /// Get a value from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(map.get(buf, &1)?, Some(&2));
    /// assert_eq!(map.get(buf, &2)?, Some(&3));
    /// assert_eq!(map.get(buf, &3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<'a, T>(&self, buf: &'a Buf, key: &T) -> Result<Option<&'a V>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: 'a + Visit,
        K::Target: Borrow<T>,
    {
        let Some(entry) = self.get_entry(buf, key)? else {
            return Ok(None);
        };

        Ok(Some(entry.1))
    }

    /// Test if the map contains the given `key`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    ///
    /// assert!(map.contains_key(buf, &1)?);
    /// assert!(map.contains_key(buf, &2)?);
    /// assert!(!map.contains_key(buf, &3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains_key<T>(&self, buf: &Buf, key: &T) -> Result<bool, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Visit,
        K::Target: Borrow<T>,
    {
        Ok(self.get_entry(buf, key)?.is_some())
    }

    /// Get an entry from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(map.get_entry(buf, &1)?, Some((&1, &2)));
    /// assert_eq!(map.get_entry(buf, &2)?, Some((&2, &3)));
    /// assert_eq!(map.get_entry(buf, &3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get_entry<'a, T>(&self, buf: &'a Buf, key: &T) -> Result<Option<(&'a K, &'a V)>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Visit,
        K::Target: Borrow<T>,
    {
        if self.displacements.is_empty() {
            return Ok(None);
        }

        let hashes = crate::phf::hashing::hash(key, &self.key);

        let displacements = |index| match self.displacements.get(index) {
            Some(entry) => Ok(Some(buf.load(entry)?)),
            None => Ok(None),
        };

        let index = crate::phf::hashing::get_custom_index(
            &hashes,
            displacements,
            self.displacements.len(),
            self.entries.len(),
        )?;

        let Some(e) = self.entries.get(index) else {
            return Ok(None);
        };

        let e = buf.load(e)?;

        if e.key.visit(buf, |v| v.borrow() == key)? {
            Ok(Some((&e.key, &e.value)))
        } else {
            Ok(None)
        }
    }
}

impl<K, V, O: Size> Clone for MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V, O: Size> Copy for MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
}
