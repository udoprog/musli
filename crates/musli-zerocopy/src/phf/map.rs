use core::borrow::Borrow;
use core::hash::Hash;

use crate::bind::Bindable;
use crate::buf::Buf;
use crate::error::Error;
use crate::pair::Pair;
use crate::phf::hashing::HashKey;
use crate::slice::Slice;
use crate::visit::Visit;
use crate::zero_copy::ZeroCopy;

/// A map bound to a [`Buf`] through [`Buf::bind`] for convenience.
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::{AlignedBuf, Pair};
///
/// let mut buf = AlignedBuf::new();
///
/// let mut map = Vec::new();
///
/// map.push(Pair::new(1, 2));
/// map.push(Pair::new(2, 3));
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
    entries: &'a [Pair<K, V>],
    displacements: &'a [Pair<u32, u32>],
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
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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

        if e.a.visit(self.buf, |v| v.borrow() == key)? {
            Ok(Some((&e.a, &e.b)))
        } else {
            Ok(None)
        }
    }
}

/// Bind a [`MapRef`] into a [`Map`].
impl<K: 'static, V: 'static> Bindable for MapRef<K, V>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    type Bound<'a> = Map<'a, K, V>;

    fn bind(self, buf: &Buf) -> Result<Self::Bound<'_>, Error> {
        Ok(Map {
            key: self.key,
            entries: buf.load(self.entries)?,
            displacements: buf.load(self.displacements)?,
            buf,
        })
    }
}

/// The reference to a map.
///
/// Constructed through [`AlignedBuf::insert_map`].
///
/// [`AlignedBuf::insert_map`]: crate::AlignedBuf::insert_map
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::{AlignedBuf, Pair};
///
/// let mut buf = AlignedBuf::new();
///
/// let mut map = Vec::new();
///
/// map.push(Pair::new(1, 2));
/// map.push(Pair::new(2, 3));
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
pub struct MapRef<K, V> {
    key: HashKey,
    entries: Slice<Pair<K, V>>,
    displacements: Slice<Pair<u32, u32>>,
}

impl<K, V> Clone for MapRef<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for MapRef<K, V> {}

impl<K, V> MapRef<K, V> {
    #[cfg(feature = "alloc")]
    pub(crate) fn new(
        key: HashKey,
        entries: Slice<Pair<K, V>>,
        displacements: Slice<Pair<u32, u32>>,
    ) -> Self {
        Self {
            key,
            entries,
            displacements,
        }
    }
}

impl<K, V> MapRef<K, V>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    /// Get a value from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
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

        let displacements = buf.load(self.displacements)?;
        let entries = buf.load(self.entries)?;

        let hashes = crate::phf::hashing::hash(key, &self.key);
        let index = crate::phf::hashing::get_index(&hashes, displacements, entries.len())?;

        let Some(e) = entries.get(index) else {
            return Ok(None);
        };

        if e.a.visit(buf, |v| v.borrow() == key)? {
            Ok(Some((&e.a, &e.b)))
        } else {
            Ok(None)
        }
    }
}
