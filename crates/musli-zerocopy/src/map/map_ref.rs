use core::borrow::Borrow;
use core::hash::Hash;

use crate::buf::{AnyRef, Buf};
use crate::error::Error;
use crate::map::hashing::HashKey;
use crate::pair::Pair;
use crate::slice_ref::SliceRef;
use crate::zero_copy::ZeroCopy;

/// The reference to a map.
#[derive(Debug, Clone, Copy)]
pub struct MapRef<K, V> {
    key: HashKey,
    entries: SliceRef<Pair<K, V>>,
    displacements: SliceRef<Pair<u32, u32>>,
}

impl<K, V> MapRef<K, V> {
    pub(crate) fn new(
        key: HashKey,
        entries: SliceRef<Pair<K, V>>,
        displacements: SliceRef<Pair<u32, u32>>,
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
    K: Copy + ZeroCopy,
    V: ZeroCopy,
{
    /// Get a value from the map.
    pub fn get<'a, T>(&self, buf: &'a Buf, key: &T) -> Result<Option<&'a V>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: 'a + AnyRef,
        K::Target: Borrow<T>,
    {
        let Some(entry) = self.get_entry(buf, key)? else {
            return Ok(None);
        };

        Ok(Some(entry.1))
    }

    /// Get an entry from within the map.
    pub fn get_entry<'a, T>(&self, buf: &'a Buf, key: &T) -> Result<Option<(&'a K, &'a V)>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: 'a + AnyRef,
        K::Target: Borrow<T>,
    {
        let displacements = buf.load(self.displacements)?;

        if displacements.is_empty() {
            return Ok(None);
        }

        let hashes = crate::map::hashing::hash(key, &self.key);

        let entries = buf.load(self.entries)?;

        let index = crate::map::hashing::get_index(&hashes, displacements, entries.len())?;

        let Some(e) = entries.get(index) else {
            return Ok(None);
        };

        if buf.load(e.a)?.borrow() == key {
            Ok(Some((&e.a, &e.b)))
        } else {
            Ok(None)
        }
    }
}
