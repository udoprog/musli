//! A set which implements a hash-set like interface, where values can be looked
//! up by keys.
//!
//! This set are implemented using a perfect hash functions, and are inserted
//! into a buffering using [`phf::store_set`].
//!
//! There's two types provided by this module:
//! * [`Set<T>`] which is a *bound* reference to a set, providing a convenient
//!   set-like access.
//! * [`SetRef<T>`] which is the *pointer* of the set. This is what you store in
//!   [`ZeroCopy`] types and is what is returned by [`phf::store_set`].
//!
//! [`phf::store_set`]: crate::phf::store_set

use core::borrow::Borrow;
use core::hash::Hash;

use crate::buf::{Bindable, Buf, Visit};
use crate::error::Error;
use crate::phf::hashing::HashKey;
use crate::phf::Entry;
use crate::pointer::{DefaultSize, Ref, Size};
use crate::ZeroCopy;

/// A set bound to a [`Buf`] through [`Buf::bind`] for convenience.
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let set = phf::store_set(&mut buf, [1, 2])?;
/// let buf = buf.into_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&1)?);
/// assert!(set.contains(&2)?);
/// assert!(!set.contains(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub struct Set<'a, T> {
    key: HashKey,
    entries: &'a [T],
    displacements: &'a [Entry<u32, u32>],
    buf: &'a Buf,
}

impl<'a, T> Set<'a, T>
where
    T: ZeroCopy,
{
    /// Get a value from the set.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::phf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let set = phf::store_set(&mut buf, [1, 2])?;
    /// let buf = buf.into_aligned();
    /// let set = buf.bind(set)?;
    ///
    /// assert!(set.contains(&1)?);
    /// assert!(set.contains(&2)?);
    /// assert!(!set.contains(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains<Q>(&self, key: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        T: Visit,
        T::Target: Borrow<Q::Target>,
    {
        if self.displacements.is_empty() {
            return Ok(false);
        }

        let hashes = crate::phf::hashing::hash(self.buf, key, &self.key)?;
        let index =
            crate::phf::hashing::get_index(&hashes, self.displacements, self.entries.len())?;

        let Some(e) = self.entries.get(index) else {
            return Ok(false);
        };

        key.visit(self.buf, |b| e.visit(self.buf, |a| a.borrow() == b))?
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
            entries: buf.load(self.entries)?,
            displacements: buf.load(self.displacements)?,
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
/// Constructed through [`phf::store_set`].
///
/// [`phf::store_set`]: crate::phf::store_set
/// [`bind()`]: crate::buf::Buf::bind
///
/// ## Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let set = phf::store_set(&mut buf, [1, 2])?;
///
/// assert!(set.contains(&buf, &1)?);
/// assert!(set.contains(&buf, &2)?);
/// assert!(!set.contains(&buf, &3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct SetRef<T, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    key: HashKey,
    entries: Ref<[T], O>,
    displacements: Ref<[Entry<u32, u32>], O>,
}

impl<T, O: Size> SetRef<T, O>
where
    T: ZeroCopy,
{
    #[cfg(feature = "alloc")]
    pub(crate) fn new(
        key: HashKey,
        entries: Ref<[T], O>,
        displacements: Ref<[Entry<u32, u32>], O>,
    ) -> Self {
        Self {
            key,
            entries,
            displacements,
        }
    }
}

impl<T, O: Size> SetRef<T, O>
where
    T: ZeroCopy,
{
    /// Get a value from the set.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::phf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let set = phf::store_set(&mut buf, [1, 2])?;
    /// let buf = buf.into_aligned();
    /// let set = buf.bind(set)?;
    ///
    /// assert!(set.contains(&1)?);
    /// assert!(set.contains(&2)?);
    /// assert!(!set.contains(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn contains<Q>(&self, buf: &Buf, key: &Q) -> Result<bool, Error>
    where
        Q: ?Sized + Visit,
        Q::Target: Eq + Hash,
        T: Visit,
        T::Target: Borrow<Q::Target>,
    {
        if self.displacements.is_empty() {
            return Ok(false);
        }

        let hashes = crate::phf::hashing::hash(buf, key, &self.key)?;

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
            return Ok(false);
        };

        let e = buf.load(e)?;
        key.visit(buf, |b| e.visit(buf, |a| a.borrow() == b))?
    }
}
