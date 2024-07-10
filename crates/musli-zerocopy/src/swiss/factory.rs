use core::hash::{Hash, Hasher};
use core::mem::size_of;

use crate::buf::{Buf, OwnedBuf, StoreBuf, Visit};
use crate::endian::ByteOrder;
use crate::error::Error;
use crate::pointer::{Ref, Size};
use crate::sip::SipHasher13;
use crate::swiss::constructor::Constructor;
use crate::swiss::map::RawTableRef;
use crate::swiss::raw::{self};
use crate::swiss::{Entry, MapRef, SetRef};
use crate::ZeroCopy;

const FIXED_SEED: u64 = 1234567890;

/// Store a [SwissTable] map into an [`OwnedBuf`].
///
/// This returns a [`MapRef`] which can be bound into a [`Map`] through the
/// [`bind()`] method for convenience.
///
/// See the [module level documentation] for more information.
///
/// [`bind()`]: crate::buf::Buf::bind
/// [`Map`]: crate::swiss::Map
/// [SwissTable]: https://abseil.io/about/design/swisstables
/// [module level documentation]: crate::swiss
///
/// # Duplicates
///
/// The caller is responsible for ensuring that no duplicate keys are provided
/// to the constructor, since the input will be used to size the table.
///
/// In the face of duplicate keys, the default accessor will return a random
/// element and the table will waste space.
///
/// Technically the elements are stored so the length is affected, and a future
/// update might make them available.
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let map = swiss::store_map(&mut buf, [(10, 1u32), (10, 2u32)])?;
/// let map = buf.bind(map)?;
///
/// assert!(matches!(map.get(&10)?, Some(&1) | Some(&2)));
/// assert_eq!(map.len(), 2);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// # Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let pairs = [
///     (buf.store_unsized("first"), 1u32),
///     (buf.store_unsized("second"), 2u32),
/// ];
///
/// let map = swiss::store_map(&mut buf, pairs)?;
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
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let map = swiss::store_map(&mut buf, [(10u64, 1u32), (20u64, 2u32)])?;
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&10u64)?, Some(&1));
/// assert_eq!(map.get(&20u64)?, Some(&2));
/// assert_eq!(map.get(&30u64)?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Storing a zero-sized type:
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let map = swiss::store_map(&mut buf, [((), ()), ((), ())])?;
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&())?, Some(&()));
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_map<K, V, I, E, O>(
    buf: &mut OwnedBuf<E, O>,
    entries: I,
) -> Result<MapRef<K, V, E, O>, Error>
where
    K: Visit + ZeroCopy,
    V: ZeroCopy,
    K::Target: Hash,
    I: IntoIterator<Item = (K, V)>,
    I::IntoIter: ExactSizeIterator,
    E: ByteOrder,
    O: Size,
{
    let (key, ctrl, buckets, bucket_mask, len) = store_raw(entries, buf, |buf, (k, v), hasher| {
        k.visit(buf, |key| key.hash(hasher))?;
        Ok(Entry::new(k, v))
    })?;

    Ok(MapRef::new(
        key,
        RawTableRef::new(ctrl, buckets, bucket_mask, len),
    ))
}

/// Store a [SwissTable] set into an [`OwnedBuf`].
///
/// This returns a [`SetRef`] which can be bound into a [`Set`] through the
/// [`bind()`] method for convenience.
///
/// See the [module level documentation] for more information.
///
/// [`bind()`]: crate::buf::Buf::bind
/// [`Set`]: crate::swiss::Set
/// [SwissTable]: https://abseil.io/about/design/swisstables
/// [module level documentation]: crate::swiss
///
/// # Duplicates
///
/// The caller is responsible for ensuring that no duplicate values are provided
/// to the constructor, since the input will be used to size the table.
///
/// In the face of duplicate values, one of the entries will be preserved at
/// random and the table will waste space.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let first = buf.store_unsized("first");
/// let second = buf.store_unsized("second");
/// let third = buf.store_unsized("third");
///
/// let set = swiss::store_set(&mut buf, [first, second])?;
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
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let set = swiss::store_set(&mut buf, [1, 2])?;
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&1)?);
/// assert!(set.contains(&2)?);
/// assert!(!set.contains(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Storing a zero-sized type:
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::swiss;
///
/// let mut buf = OwnedBuf::new();
///
/// let set = swiss::store_set(&mut buf, [(), ()])?;
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&())?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_set<T, I, S>(
    buf: &mut S,
    entries: I,
) -> Result<SetRef<T, S::ByteOrder, S::Size>, Error>
where
    T: Visit + ZeroCopy,
    T::Target: Hash,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    S: ?Sized + StoreBuf,
{
    let (key, ctrl, buckets, bucket_mask, len) = store_raw(entries, buf, |buf, v, hasher| {
        v.visit(buf, |key| key.hash(hasher))?;
        Ok(v)
    })?;

    Ok(SetRef::new(
        key,
        RawTableRef::new(ctrl, buckets, bucket_mask, len),
    ))
}

// Output from storing raw values.
type Raw<U, E, O> = (u64, Ref<[u8], E, O>, Ref<[U], E, O>, usize, usize);

// Raw store function which is capable of storing any value using a hashing
// adapter.
fn store_raw<T, U, I, S>(
    entries: I,
    buf: &mut S,
    hash: fn(&Buf, T, &mut SipHasher13) -> Result<U, Error>,
) -> Result<Raw<U, S::ByteOrder, S::Size>, Error>
where
    U: ZeroCopy,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    S: ?Sized + StoreBuf,
{
    let entries = entries.into_iter();
    let key = FIXED_SEED;

    let Some(buckets) = raw::capacity_to_buckets(entries.len()) else {
        panic!("Capacity overflow");
    };

    let ctrl_len = buckets + size_of::<raw::Group>();
    let ctrl_align = raw::Group::WIDTH;

    debug_assert!(ctrl_align.is_power_of_two());

    buf.next_offset_with_and_reserve(ctrl_align, ctrl_len);
    let ctrl_ptr = buf.len();

    // All ones indicates that the table is empty, since the ctrl byte for empty
    // buckets is 1111_1111.
    buf.fill(raw::EMPTY, ctrl_len + size_of::<raw::Group>());

    let base_ptr = buf.next_offset::<U>();
    buf.fill(0, size_of::<T>().wrapping_mul(buckets));

    let (bucket_mask, len) = {
        buf.align_in_place();
        let mut table = Constructor::<U, _>::with_buf(buf, ctrl_ptr, base_ptr, buckets);

        for v in entries {
            let mut hasher = SipHasher13::new_with_keys(0, key);
            let v = hash(table.buf(), v, &mut hasher)?;
            let hash = hasher.finish();
            table.insert(hash, &v)?;
        }

        (table.bucket_mask(), table.len())
    };

    let ctrl = Ref::with_metadata(ctrl_ptr, ctrl_len);
    let buckets = Ref::with_metadata(base_ptr, buckets);
    Ok((key, ctrl, buckets, bucket_mask, len))
}
