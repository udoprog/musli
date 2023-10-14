use core::hash::{Hash, Hasher};
use core::ptr;

use alloc::vec::Vec;

use crate::buf::Visit;
use crate::error::Error;
use crate::pointer::Size;
use crate::sip::SipHasher13;
use crate::swiss::raw::RawTable;
use crate::swiss::{Entry, MapRef, RawOption, RawTableRef, SetRef};
use crate::AlignedBuf;
use crate::ZeroCopy;

const FIXED_SEED: u64 = 1234567890;

/// Store a map based on a perfect hash function into the [`AlignedBuf`].
///
/// This will utilize a perfect hash functions derived from the [`hashbrown`
/// crate] to construct a persistent hash Map.
///
/// This returns a [`MapRef`] which can be bound into a [`Map`] through the
/// [`bind()`] method for convenience.
///
/// [`hashbrown` crate]: https://crates.io/crates/hashbrown
/// [`Map`]: crate::swiss::Map
/// [`bind()`]: crate::buf::Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let pairs = [
///     (buf.store_unsized("first"), 1u32),
///     (buf.store_unsized("second"), 2u32),
/// ];
///
/// let map = swiss::store_map(&mut buf, pairs)?;
/// let buf = buf.as_aligned();
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get("first")?, Some(&1));
/// assert_eq!(map.get("second")?, Some(&2));
/// assert_eq!(map.get("third")?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Using non-references as keys:
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let map = swiss::store_map(&mut buf, [(10u64, 1u32), (20u64, 2u32)])?;
/// let buf = buf.as_aligned();
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&10u64)?, Some(&1));
/// assert_eq!(map.get(&20u64)?, Some(&2));
/// assert_eq!(map.get(&30u64)?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_map<K, V, I, O: Size>(
    buf: &mut AlignedBuf<O>,
    entries: I,
) -> Result<MapRef<K, V, O>, Error>
where
    K: Visit + ZeroCopy,
    V: ZeroCopy,
    K::Target: Hash,
    I: IntoIterator<Item = (K, V)>,
    I::IntoIter: ExactSizeIterator,
{
    let entries = entries.into_iter();
    let mut table = RawTable::with_capacity(entries.len() + 64);

    let key = FIXED_SEED;

    {
        let buf = buf.as_aligned();

        for (k, v) in entries {
            let mut hasher = SipHasher13::new_with_keys(0, key);
            k.visit(buf, |key| key.hash(&mut hasher))?;
            let hash = hasher.finish();
            table.insert(hash, Entry::new(k, v));
        }
    }

    let bucket_mask = table.bucket_mask();
    let ctrl = table.control_bytes();

    let ctrl = buf.store_unsized(ctrl);

    let mut buckets = Vec::with_capacity(table.buckets());

    for _ in 0..table.buckets() {
        buckets.push(RawOption::None);
    }

    unsafe {
        for bucket in table.iter() {
            // SAFETY: ZeroType types can be bitwise copied.
            buckets[bucket.index()] = RawOption::Some(ptr::read_unaligned(bucket.as_ptr()));
        }
    }

    let buckets = buf.store_slice(&buckets);

    Ok(MapRef::new(
        key,
        RawTableRef::new(ctrl, buckets, bucket_mask),
    ))
}

/// Store a set based on a perfect hash function into the [`AlignedBuf`].
///
/// This will utilize a perfect hash functions derived from the [`hashbrown`
/// crate] to construct a persistent hash Map.
///
/// This returns a [`SetRef`] which can be bound into a [`Set`] through the
/// [`bind()`] method for convenience.
///
/// [`hashbrown` crate]: https://crates.io/crates/hashbrown
/// [`Set`]: crate::swiss::Set
/// [`bind()`]: crate::buf::Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let pairs = [buf.store_unsized("first"), buf.store_unsized("second")];
///
/// let set = swiss::store_set(&mut buf, pairs)?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains("first")?);
/// assert!(set.contains("second")?);
/// assert!(!set.contains("third")?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Using non-references as keys:
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = AlignedBuf::new();
///
/// let set = swiss::store_set(&mut buf, [10u64, 20u64])?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&10u64)?);
/// assert!(set.contains(&20u64)?);
/// assert!(!set.contains(&30u64)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_set<T, I, O: Size>(buf: &mut AlignedBuf<O>, entries: I) -> Result<SetRef<T, O>, Error>
where
    T: Visit + ZeroCopy,
    T::Target: Hash,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let entries = entries.into_iter();
    let mut table = RawTable::with_capacity(entries.len() + 64);

    let key = FIXED_SEED;

    {
        let buf = buf.as_aligned();

        for entry in entries {
            let mut hasher = SipHasher13::new_with_keys(0, key);
            entry.visit(buf, |key| key.hash(&mut hasher))?;
            let hash = hasher.finish();
            table.insert(hash, entry);
        }
    }

    let bucket_mask = table.bucket_mask();
    let ctrl = table.control_bytes();

    let ctrl = buf.store_unsized(ctrl);

    let mut buckets = Vec::with_capacity(table.buckets());

    for _ in 0..table.buckets() {
        buckets.push(RawOption::None);
    }

    unsafe {
        for bucket in table.iter() {
            // SAFETY: ZeroType types can be bitwise copied.
            buckets[bucket.index()] = RawOption::Some(ptr::read_unaligned(bucket.as_ptr()));
        }
    }

    let buckets = buf.store_slice(&buckets);

    Ok(SetRef::new(
        key,
        RawTableRef::new(ctrl, buckets, bucket_mask),
    ))
}
