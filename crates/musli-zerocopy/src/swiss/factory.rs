use core::hash::{Hash, Hasher};
use core::mem::size_of;

use crate::buf::{AlignedBuf, Buf, Visit};
use crate::error::Error;
use crate::pointer::{Size, Slice, Unsized};
use crate::sip::SipHasher13;
use crate::swiss::constructor::Constructor;
use crate::swiss::map::RawTableRef;
use crate::swiss::raw::{self};
use crate::swiss::{Entry, MapRef, SetRef};
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
    let (key, ctrl, buckets, bucket_mask) = store_raw(entries, buf, |buf, (k, v), hasher| {
        k.visit(buf, |key| key.hash(hasher))?;
        Ok(Entry::new(k, v))
    })?;

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
/// let first = buf.store_unsized("first");
/// let second = buf.store_unsized("second");
/// let third = buf.store_unsized("third");
///
/// let set = swiss::store_set(&mut buf, [first, second])?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains("first")?);
/// assert!(set.contains(&first)?);
/// assert!(set.contains("second")?);
/// assert!(set.contains(&second)?);
/// assert!(!set.contains("third")?);
/// assert!(!set.contains(&third)?);
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
/// let set = swiss::store_set(&mut buf, [1, 2])?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&1)?);
/// assert!(set.contains(&2)?);
/// assert!(!set.contains(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_set<T, I, O: Size>(buf: &mut AlignedBuf<O>, entries: I) -> Result<SetRef<T, O>, Error>
where
    T: Visit + ZeroCopy,
    T::Target: Hash,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let (key, ctrl, buckets, bucket_mask) = store_raw(entries, buf, |buf, v, hasher| {
        v.visit(buf, |key| key.hash(hasher))?;
        Ok(v)
    })?;

    Ok(SetRef::new(
        key,
        RawTableRef::new(ctrl, buckets, bucket_mask),
    ))
}

// Output from storing raw values.
type Raw<U, O> = (u64, Unsized<[u8], O>, Slice<U, O>, usize);

// Raw store function which is capable of storing any value using a hashing
// adapter.
fn store_raw<T, U, I, O: Size>(
    entries: I,
    buf: &mut AlignedBuf<O>,
    hash: fn(&Buf, T, &mut SipHasher13) -> Result<U, Error>,
) -> Result<Raw<U, O>, Error>
where
    U: ZeroCopy,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let entries = entries.into_iter();
    let key = FIXED_SEED;

    let Some(buckets) = raw::capacity_to_buckets(entries.len()) else {
        panic!("Capacity overflow");
    };

    let ctrl_len = buckets + size_of::<raw::Group>();
    let ctrl_align = raw::Group::WIDTH;

    debug_assert!(ctrl_align.is_power_of_two());

    let ctrl_ptr = unsafe { buf.next_offset_with(ctrl_align, ctrl_len) };
    // All ones indicates that the table is empty, since the ctrl byte for empty
    // buckets is 1111_1111.
    buf.fill(raw::EMPTY, ctrl_len + size_of::<raw::Group>());

    let base_ptr = buf.next_offset::<U>();
    buf.fill(0, size_of::<T>().wrapping_mul(buckets));

    let (buckets, bucket_mask) = {
        buf.as_aligned();

        let mut table = unsafe { Constructor::<U, O>::with_buf(buf, ctrl_ptr, base_ptr, buckets) };

        for v in entries {
            let mut hasher = SipHasher13::new_with_keys(0, key);
            let v = hash(table.buf(), v, &mut hasher)?;
            let hash = hasher.finish();
            table.insert(hash, v)?;
        }

        (table.buckets(), table.bucket_mask())
    };

    let ctrl = Unsized::new(ctrl_ptr, ctrl_len);
    let buckets = Slice::new(base_ptr, buckets);
    Ok((key, ctrl, buckets, bucket_mask))
}
