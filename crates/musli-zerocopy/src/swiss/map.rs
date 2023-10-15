//! A map which implements a hash-map like interface, where values can be looked
//! up by keys.
//!
//! This map are implemented using a perfect hash functions, and are inserted
//! into a buffering using [`phf::store_map`].
//!
//! There's two types provided by this module:
//! * [`Map<K, V>`] which is a *bound* reference to a map, providing a
//!   convenient map-like access.
//! * [`MapRef<K, V>`] which is the *pointer* of the map. This is what you store
//!   in [`ZeroCopy`] types and is what is returned by [`phf::store_map`].
//!
//! [`phf::store_map`]: crate::phf::store_map

use core::borrow::Borrow;
use core::convert::identity as likely;
use core::hash::{Hash, Hasher};
use core::mem::size_of;

use crate::buf::{Bindable, Buf, Visit};
use crate::error::{Error, ErrorKind};
use crate::pointer::{DefaultSize, Size, Slice, Unsized};
use crate::sip::SipHasher13;
use crate::swiss::raw::{h2, probe_seq, Group};
use crate::swiss::Entry;
use crate::ZeroCopy;

/// A map bound to a [`Buf`] through [`Buf::bind`] for convenience.
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = AlignedBuf::new();
///
/// let map = phf::store_map(&mut buf, [(1, 2), (2, 3)])?;
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
    key: u64,
    table: RawTable<'a, Entry<K, V>>,
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
    /// use musli_zerocopy::phf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let map = phf::store_map(&mut buf, [(1, 2), (2, 3)])?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get(&1)?, Some(&2));
    /// assert_eq!(map.get(&2)?, Some(&3));
    /// assert_eq!(map.get(&3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Result<Option<&V>, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        K: Visit,
        K::Target: Borrow<Q::Target>,
    {
        let hash = key.visit(self.buf, |k| self.hash(k))?;

        let entry = self.table.find(hash, |e| {
            key.visit(self.buf, |b| e.key.visit(self.buf, |a| a.borrow() == b))?
        })?;

        if let Some(entry) = entry {
            Ok(Some(&entry.value))
        } else {
            Ok(None)
        }
    }

    /// Test if the map contains the given `key`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::phf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let map = phf::store_map(&mut buf, [(1, 2), (2, 3)])?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert!(map.contains_key(&1)?);
    /// assert!(map.contains_key(&2)?);
    /// assert!(!map.contains_key(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        K: Visit,
        K::Target: Borrow<Q::Target>,
    {
        let hash = key.visit(self.buf, |k| self.hash(k))?;

        let entry = self.table.find(hash, |e| {
            key.visit(self.buf, |b| e.key.visit(self.buf, |a| a.borrow() == b))?
        })?;

        Ok(entry.is_some())
    }

    fn hash<H: ?Sized>(&self, value: &H) -> u64
    where
        H: Hash,
    {
        let mut hasher = SipHasher13::new_with_keys(0, self.key);
        value.hash(&mut hasher);
        hasher.finish()
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
            table: self.table.bind(buf)?,
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
/// Constructed through [`phf::store_map`].
///
/// [`phf::store_map`]: crate::phf::store_map
/// [`bind()`]: crate::buf::Buf::bind
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = AlignedBuf::new();
///
/// let map = phf::store_map(&mut buf, [(1, 2), (2, 3)])?;
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
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct MapRef<K, V, O: Size = DefaultSize>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    key: u64,
    table: RawTableRef<Entry<K, V>, O>,
}

impl<K, V, O: Size> MapRef<K, V, O>
where
    K: ZeroCopy,
    V: ZeroCopy,
{
    pub(crate) fn new(key: u64, table: RawTableRef<Entry<K, V>, O>) -> Self {
        Self { key, table }
    }

    /// Get a value from the map.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::phf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let map = phf::store_map(&mut buf, [(1, 2), (2, 3)])?;
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(map.get(buf, &1)?, Some(&2));
    /// assert_eq!(map.get(buf, &2)?, Some(&3));
    /// assert_eq!(map.get(buf, &3)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<'a, Q>(&self, buf: &'a Buf, key: &Q) -> Result<Option<&'a V>, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        K: 'a + Visit,
        K::Target: Borrow<Q::Target>,
    {
        let hash = key.visit(buf, |k| self.hash(k))?;

        let entry = self.table.find(buf, hash, |e| {
            key.visit(buf, |b| e.key.visit(buf, |a| a.borrow() == b))?
        })?;

        if let Some(entry) = entry {
            Ok(Some(&entry.value))
        } else {
            Ok(None)
        }
    }

    /// Test if the map contains the given `key`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::swiss;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let map = swiss::store_map(&mut buf, [(1, 2), (2, 3)])?;
    /// let buf = buf.as_aligned();
    ///
    /// assert!(map.contains_key(buf, &1)?);
    /// assert!(map.contains_key(buf, &2)?);
    /// assert!(!map.contains_key(buf, &3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains_key<Q>(&self, buf: &Buf, key: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        K: Visit,
        K::Target: Borrow<Q::Target>,
    {
        let hash = key.visit(buf, |key| self.hash(key))?;

        let entry = self.table.find(buf, hash, |e| {
            key.visit(buf, |b| e.key.visit(buf, |a| a.borrow() == b))?
        })?;

        Ok(entry.is_some())
    }

    fn hash<H: ?Sized>(&self, value: &H) -> u64
    where
        H: Hash,
    {
        let mut hasher = SipHasher13::new_with_keys(0, self.key);
        value.hash(&mut hasher);
        hasher.finish()
    }
}

pub(crate) struct RawTable<'a, T> {
    ctrl: &'a [u8],
    entries: &'a [T],
    bucket_mask: usize,
}

impl<'a, T> RawTable<'a, T> {
    /// Searches for an element in the table.
    #[inline]
    pub(crate) fn find(
        &self,
        hash: u64,
        mut eq: impl FnMut(&T) -> Result<bool, Error>,
    ) -> Result<Option<&'a T>, Error> {
        let result = self.find_inner(hash, &mut |index| {
            let entry = self.entry(index)?;
            eq(entry)
        })?;

        Ok(match result {
            Some(index) => Some(self.entry(index)?),
            None => None,
        })
    }

    fn entry(&self, index: usize) -> Result<&'a T, Error> {
        let Some(entry) = self.entries.get(index) else {
            return Err(Error::new(ErrorKind::IndexOutOfBounds {
                index,
                len: self.entries.len(),
            }));
        };

        Ok(entry)
    }

    /// Searches for an element in a table, returning the `index` of the found
    /// element.
    #[inline(always)]
    fn find_inner(
        &self,
        hash: u64,
        eq: &mut dyn FnMut(usize) -> Result<bool, Error>,
    ) -> Result<Option<usize>, Error> {
        let h2_hash = h2(hash);
        let mut probe_seq = probe_seq(self.bucket_mask, hash);

        loop {
            let range = probe_seq.pos..probe_seq.pos.wrapping_add(size_of::<Group>());

            let Some(bytes) = self.ctrl.get(range.clone()) else {
                return Err(Error::new(ErrorKind::ControlRangeOutOfBounds {
                    range,
                    len: self.ctrl.len(),
                }));
            };

            // SAFETY: We've made sure to provide this load with a buffer of the
            // appropriate size.
            let group = unsafe { Group::load(bytes.as_ptr()) };

            for bit in group.match_byte(h2_hash) {
                // This is the same as `(probe_seq.pos + bit) % self.buckets()` because the number
                // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
                let index = (probe_seq.pos + bit) & self.bucket_mask;

                if likely(eq(index)?) {
                    return Ok(Some(index));
                }
            }

            if likely(group.match_empty().any_bit_set()) {
                return Ok(None);
            }

            probe_seq.move_next(self.bucket_mask);
        }
    }
}

#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub(crate) struct RawTableRef<T, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    ctrl: Unsized<[u8], O>,
    entries: Slice<T, O>,
    bucket_mask: usize,
}

impl<T, O: Size> RawTableRef<T, O>
where
    T: ZeroCopy,
{
    pub(crate) fn new(ctrl: Unsized<[u8], O>, entries: Slice<T, O>, bucket_mask: usize) -> Self {
        Self {
            ctrl,
            entries,
            bucket_mask,
        }
    }

    pub(crate) fn bind<'buf>(&self, buf: &'buf Buf) -> Result<RawTable<'buf, T>, Error> {
        Ok(RawTable {
            ctrl: buf.load(self.ctrl)?,
            entries: buf.load(self.entries)?,
            bucket_mask: self.bucket_mask,
        })
    }
}

impl<T, O: Size> RawTableRef<T, O>
where
    T: ZeroCopy,
{
    /// Searches for an element in the table.
    #[inline]
    pub(crate) fn find<'buf>(
        &self,
        buf: &'buf Buf,
        hash: u64,
        mut eq: impl FnMut(&T) -> Result<bool, Error>,
    ) -> Result<Option<&'buf T>, Error> {
        let result = self.find_inner(buf, hash, &mut |index| eq(self.entry(index, buf)?))?;

        Ok(match result {
            Some(index) => Some(self.entry(index, buf)?),
            None => None,
        })
    }

    fn entry<'buf>(&self, index: usize, buf: &'buf Buf) -> Result<&'buf T, Error> {
        let Some(entry) = self.entries.get(index) else {
            return Err(Error::new(ErrorKind::IndexOutOfBounds {
                index,
                len: self.entries.len(),
            }));
        };

        buf.load(entry)
    }

    /// Searches for an element in a table, returning the `index` of the found
    /// element.
    #[inline(always)]
    fn find_inner(
        &self,
        buf: &Buf,
        hash: u64,
        eq: &mut dyn FnMut(usize) -> Result<bool, Error>,
    ) -> Result<Option<usize>, Error> {
        let h2_hash = h2(hash);
        let mut probe_seq = probe_seq(self.bucket_mask, hash);

        let ctrl = buf.load(self.ctrl)?;

        loop {
            let range = probe_seq.pos..probe_seq.pos.wrapping_add(size_of::<Group>());

            let Some(bytes) = ctrl.get(range.clone()) else {
                return Err(Error::new(ErrorKind::ControlRangeOutOfBounds {
                    range,
                    len: self.ctrl.len(),
                }));
            };

            let group = unsafe { Group::load(bytes.as_ptr()) };

            for bit in group.match_byte(h2_hash) {
                // This is the same as `(probe_seq.pos + bit) % self.buckets()` because the number
                // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
                let index = (probe_seq.pos + bit) & self.bucket_mask;

                if likely(eq(index)?) {
                    return Ok(Some(index));
                }
            }

            if likely(group.match_empty().any_bit_set()) {
                return Ok(None);
            }

            probe_seq.move_next(self.bucket_mask);
        }
    }
}
