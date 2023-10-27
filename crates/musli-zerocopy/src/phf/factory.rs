#![allow(clippy::type_complexity)]

use core::hash::Hash;

use crate::buf::{StoreBuf, Visit};
use crate::error::Error;
use crate::phf::hashing::HashKey;
use crate::phf::{Entry, MapRef, SetRef};
use crate::Ref;
use crate::ZeroCopy;

/// Store a map based on a perfect hash function into a buffer.
///
/// This will utilize a perfect hash functions derived from the [`phf` crate] to
/// construct a persistent hash set.
///
/// This returns a [`MapRef`] which can be bound into a [`Map`] through the
/// [`bind()`] method for convenience.
///
/// [`phf` crate]: https://crates.io/crates/phf
/// [`Map`]: crate::phf::Map
/// [`bind()`]: crate::buf::Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let first = buf.store_unsized("first");
/// let second = buf.store_unsized("second");
///
/// let map = phf::store_map(&mut buf, [(first, 1u32), (second, 2u32)])?;
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get("first")?, Some(&1));
/// assert_eq!(map.get(&first)?, Some(&1));
/// assert_eq!(map.get("second")?, Some(&2));
/// assert_eq!(map.get(&second)?, Some(&2));
/// assert_eq!(map.get("third")?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Using non-references as keys:
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let map = phf::store_map(&mut buf, [(10u64, 1), (20u64, 2)])?;
/// let map = buf.bind(map)?;
///
/// assert_eq!(map.get(&10u64)?, Some(&1));
/// assert_eq!(map.get(&20u64)?, Some(&2));
/// assert_eq!(map.get(&30u64)?, None);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_map<K, V, S, I>(
    buf: &mut S,
    entries: I,
) -> Result<MapRef<K, V, S::ByteOrder, S::Size>, Error>
where
    K: Visit + ZeroCopy,
    V: ZeroCopy,
    K::Target: Hash,
    S: ?Sized + StoreBuf,
    I: IntoIterator<Item = (K, V)>,
    I::IntoIter: ExactSizeIterator,
{
    let entries = entries.into_iter().map(|(k, v)| Entry::new(k, v));
    let (key, entries, displacements) = store_raw(buf, entries, |entry| &entry.key)?;
    Ok(MapRef::new(key, entries, displacements))
}

/// Store a set based on a perfect hash function into a buffer.
///
/// This will utilize a perfect hash functions derived from the [`phf` crate] to
/// construct a persistent hash map.
///
/// This returns a [`SetRef`] which can be bound into a [`Set`] through the
/// [`bind()`] method for convenience.
///
/// [`phf` crate]: https://crates.io/crates/phf
/// [`Set`]: crate::phf::Set
/// [`bind()`]: crate::buf::Buf::bind
///
/// # Examples
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let first = buf.store_unsized("first");
/// let second = buf.store_unsized("second");
/// let third = buf.store_unsized("third");
///
/// let set = phf::store_set(&mut buf, [first, second])?;
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
/// use musli_zerocopy::phf;
///
/// let mut buf = OwnedBuf::new();
///
/// let set = phf::store_set(&mut buf, [1, 2])?;
/// let set = buf.bind(set)?;
///
/// assert!(set.contains(&1)?);
/// assert!(set.contains(&2)?);
/// assert!(!set.contains(&3)?);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn store_set<S, I>(
    buf: &mut S,
    entries: I,
) -> Result<SetRef<I::Item, S::ByteOrder, S::Size>, Error>
where
    S: ?Sized + StoreBuf,
    I: IntoIterator,
    I::Item: Visit + ZeroCopy,
    <I::Item as Visit>::Target: Hash,
    I::IntoIter: ExactSizeIterator,
{
    let (key, entries, displacements) = store_raw(buf, entries, |entry| entry)?;
    Ok(SetRef::new(key, entries, displacements))
}

fn store_raw<K, I, S, F>(
    buf: &mut S,
    entries: I,
    access: F,
) -> Result<
    (
        HashKey,
        Ref<[I::Item], S::ByteOrder, S::Size>,
        Ref<[Entry<u32, u32>], S::ByteOrder, S::Size>,
    ),
    Error,
>
where
    K: Visit + ZeroCopy,
    K::Target: Hash,
    I: IntoIterator,
    I::Item: ZeroCopy,
    I::IntoIter: ExactSizeIterator,
    S: ?Sized + StoreBuf,
    F: Fn(&I::Item) -> &K,
{
    let offset = buf.next_offset::<I::Item>();
    let iter = entries.into_iter();
    let len = iter.len();

    for value in iter {
        buf.store(&value);
    }

    let entries: Ref<[I::Item], _, _> = Ref::with_metadata(offset, len);

    let offset = buf.next_offset::<Entry<u32, u32>>();
    let len = crate::phf::generator::displacements_len(entries.len());

    for _ in 0..len {
        buf.store(&Entry::new(0, 0));
    }

    let displacements = Ref::with_metadata(offset, len);

    let mut hash_state = {
        buf.align_in_place();
        crate::phf::generator::generate_hash(buf.as_mut_buf(), &entries, &displacements, access)?
    };

    for a in 0..hash_state.map.len() {
        loop {
            let b = hash_state.map[a];

            if hash_state.map[a] != a {
                let Some(ref_a) = entries.get(a) else {
                    panic!("Missing entry at index {a}");
                };

                let Some(ref_b) = entries.get(b) else {
                    panic!("Missing entry at index {b}");
                };

                buf.swap(ref_a, ref_b)?;
                hash_state.map.swap(a, b);
                continue;
            }

            break;
        }
    }

    Ok((hash_state.key, entries, displacements))
}
