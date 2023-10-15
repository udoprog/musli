//! A set which implements a hash-set like interface, where values can be looked
//! up by keys.
//!
//! This set are implemented using a perfect hash functions, and are inserted
//! into a buffering using [`swiss::store_set`].
//!
//! There's two types provided by this module:
//! * [`Set<T>`] which is a *bound* reference to a set, providing a convenient
//!   set-like access.
//! * [`SetRef<T>`] which is the *pointer* of the set. This is what you store in
//!   [`ZeroCopy`] types and is what is returned by [`swiss::store_set`].
//!
//! [`swiss::store_set`]: crate::swiss::store_set

use core::borrow::Borrow;
use core::hash::{Hash, Hasher};

use crate::buf::{Bindable, Buf, Visit};
use crate::error::Error;
use crate::pointer::{DefaultSize, Size};
use crate::sip::SipHasher13;
use crate::swiss::map::{RawTable, RawTableRef};
use crate::ZeroCopy;

/// A set bound to a [`Buf`] through [`Buf::bind`] for convenience.
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let set = swiss::store_set(&mut buf, [1, 2])?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&1)?);
/// assert!(set.contains(&2)?);
/// assert!(!set.contains(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub struct Set<'a, T> {
    key: u64,
    table: RawTable<'a, T>,
    buf: &'a Buf,
}

impl<'a, T> Set<'a, T>
where
    T: ZeroCopy,
{
    /// Test if the set contains the given `value`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::swiss;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let set = swiss::store_set(&mut buf, [1, 2])?;
    /// let buf = buf.as_aligned();
    /// let set = buf.bind(set)?;
    ///
    /// assert!(set.contains(&1)?);
    /// assert!(set.contains(&2)?);
    /// assert!(!set.contains(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains<Q>(&self, value: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        T: Visit,
        T::Target: Borrow<Q::Target>,
    {
        let hash = value.visit(self.buf, |k| self.hash(k))?;

        let entry = self.table.find(hash, |e| {
            value.visit(self.buf, |b| e.visit(self.buf, |a| a.borrow() == b))?
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

/// Bind a [`SetRef`] into a [`Set`].
impl<T, O: Size> Bindable for SetRef<T, O>
where
    T: ZeroCopy,
{
    type Bound<'a> = Set<'a, T> where Self: 'a;

    fn bind(self, buf: &Buf) -> Result<Self::Bound<'_>, Error> {
        Ok(Set {
            key: self.key,
            table: self.table.bind(buf)?,
            buf,
        })
    }
}

/// A stored reference to a set.
///
/// Note that operating over the methods provided in [`SetRef`] does not demand
/// that the entire contents of the set is validated as would be the case when
/// [`bind()`] is used and might result in better performance if the data is
/// infrequently accessed.
///
/// Constructed through [`swiss::store_set`].
///
/// [`swiss::store_set`]: crate::swiss::store_set
/// [`bind()`]: crate::buf::Buf::bind
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let set = swiss::store_set(&mut buf, [1, 2])?;
/// let buf = buf.as_aligned();
///
/// assert!(set.contains(buf, &1)?);
/// assert!(!set.contains(buf, &3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct SetRef<T, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    key: u64,
    table: RawTableRef<T, O>,
}

impl<T, O: Size> SetRef<T, O>
where
    T: ZeroCopy,
{
    #[cfg(feature = "alloc")]
    pub(crate) fn new(key: u64, table: RawTableRef<T, O>) -> Self {
        Self { key, table }
    }

    /// Test if the set contains the given `key`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::swiss;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let set = swiss::store_set(&mut buf, [1, 2])?;
    /// let buf = buf.as_aligned();
    ///
    /// assert!(set.contains(buf, &1)?);
    /// assert!(set.contains(buf, &2)?);
    /// assert!(!set.contains(buf, &3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains<Q>(&self, buf: &Buf, key: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        T: Visit,
        T::Target: Borrow<Q::Target>,
    {
        let hash = key.visit(buf, |key| self.hash(key))?;

        let entry = self.table.find(buf, hash, |e| {
            key.visit(buf, |b| e.visit(buf, |a| a.borrow() == b))?
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
