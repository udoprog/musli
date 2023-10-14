use core::hash::Hash;

use alloc::vec::Vec;

use crate::buf::Visit;
use crate::error::Error;
use crate::phf::{Entry, MapRef, SetRef};
use crate::pointer::Size;
use crate::AlignedBuf;
use crate::ZeroCopy;

/// Store a map based on a perfect hash function into the [`AlignedBuf`].
///
/// This will utilize a perfect hash functions derived from the [`phf`
/// crate] to construct a persistent hash set.
///
/// This returns a [`SetRef`] which can be bound into a [`Set`] through the
/// [`bind()`] method for convenience.
///
/// [`phf` crate]: https://crates.io/crates/phf
/// [`Set`]: crate::set::Set
/// [`bind()`]: Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf::{self, Entry};
///
/// let mut buf = AlignedBuf::new();
///
/// let mut pairs = Vec::new();
///
/// pairs.push(Entry::new(buf.store_unsized("first"), 1u32));
/// pairs.push(Entry::new(buf.store_unsized("second"), 2u32));
///
/// let map = phf::store_map(&mut buf, &mut pairs)?;
/// let buf = buf.as_aligned();
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&"first")?, Some(&1));
/// assert_eq!(map.get(&"second")?, Some(&2));
/// assert_eq!(map.get(&"third")?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Using non-references as keys:
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf::{self, Entry};
///
/// let mut buf = AlignedBuf::new();
///
/// let mut pairs = Vec::new();
///
/// pairs.push(Entry::new(10u64, 1u32));
/// pairs.push(Entry::new(20u64, 2u32));
///
/// let map = phf::store_map(&mut buf, &mut pairs)?;
/// let buf = buf.as_aligned();
///
/// assert_eq!(map.get(buf, &10u64)?, Some(&1));
/// assert_eq!(map.get(buf, &20u64)?, Some(&2));
/// assert_eq!(map.get(buf, &30u64)?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_map<K, V, O: Size>(
    buf: &mut AlignedBuf<O>,
    entries: &mut [Entry<K, V>],
) -> Result<MapRef<K, V, O>, Error>
where
    K: Visit + ZeroCopy,
    V: ZeroCopy,
    K::Target: Hash,
{
    let mut hash_state = {
        let buf = buf.as_aligned();
        crate::phf::generator::generate_hash(buf, entries, |entry| &entry.key)?
    };

    for a in 0..hash_state.map.len() {
        loop {
            let b = hash_state.map[a];

            if hash_state.map[a] != a {
                entries.swap(a, b);
                hash_state.map.swap(a, b);
                continue;
            }

            break;
        }
    }

    let entries = buf.store_slice(entries);

    let mut displacements = Vec::new();

    for (key, value) in hash_state.displacements {
        displacements.push(Entry::new(key, value));
    }

    let displacements = buf.store_slice(&displacements);
    Ok(MapRef::new(hash_state.key, entries, displacements))
}

/// Store a set based on a perfect hash function into the [`AlignedBuf`].
///
/// This will utilize a perfect hash functions derived from the [`phf`
/// crate] to construct a persistent hash map.
///
/// This returns a [`MapRef`] which can be bound into a [`Map`] through the
/// [`bind()`] method for convenience.
///
/// [`phf` crate]: https://crates.io/crates/phf
/// [`Map`]: crate::map::Map
/// [`bind()`]: Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = AlignedBuf::new();
///
/// let mut values = [
///     buf.store_unsized("first"),
///     buf.store_unsized("second"),
/// ];
///
/// let set = phf::store_set(&mut buf, &mut values)?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&"first")?);
/// assert!(set.contains(&"second")?);
/// assert!(!set.contains(&"third")?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Using non-references as keys:
///
/// ```
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = AlignedBuf::new();
///
/// let mut values = [10u64, 20u64];
///
/// let set = phf::store_set(&mut buf, &mut values)?;
/// let buf = buf.as_aligned();
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&10u64)?);
/// assert!(set.contains(&20u64)?);
/// assert!(!set.contains(&30u64)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_set<T, O: Size>(
    buf: &mut AlignedBuf<O>,
    entries: &mut [T],
) -> Result<SetRef<T, O>, Error>
where
    T: Visit + ZeroCopy,
    T::Target: Hash,
{
    let mut hash_state = {
        let buf = buf.as_aligned();
        crate::phf::generator::generate_hash(buf, entries, |value| value)?
    };

    for a in 0..hash_state.map.len() {
        loop {
            let b = hash_state.map[a];

            if hash_state.map[a] != a {
                entries.swap(a, b);
                hash_state.map.swap(a, b);
                continue;
            }

            break;
        }
    }

    let entries = buf.store_slice(entries);

    let mut displacements = Vec::new();

    for (key, value) in hash_state.displacements {
        displacements.push(Entry::new(key, value));
    }

    let displacements = buf.store_slice(&displacements);
    Ok(SetRef::new(hash_state.key, entries, displacements))
}
