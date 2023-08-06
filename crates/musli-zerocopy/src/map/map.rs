use core::borrow::Borrow;
use core::hash::Hash;

use crate::buf::Buf;
use crate::error::Error;
use crate::map::hashing::HashKey;
use crate::traits::{Bind, Read, Size};
use crate::{Slice, SliceRef};

/// A constant size map.
pub struct Map<'a, K: 'a, V: 'a> {
    key: HashKey,
    displacements: Slice<'a, (u32, u32)>,
    entries: Slice<'a, (K, V)>,
}

impl<'a, K: 'a, V: 'a> Map<'a, K, V> {
    /// Get a value from the map.
    pub fn get<T>(&self, key: &T) -> Result<Option<V>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Borrow<T> + Size + Read<'a>,
        V: Size + Read<'a>,
    {
        Ok(self.get_entry(key)?.map(|e| e.1))
    }

    /// Get an entry from the map.
    pub fn get_entry<T>(&self, key: &T) -> Result<Option<(K, V)>, Error>
    where
        T: ?Sized + Eq + Hash,
        K: Borrow<T> + Size + Read<'a>,
        V: Size + Read<'a>,
    {
        if self.displacements.is_empty() {
            return Ok(None);
        }

        let hashes = crate::map::hashing::hash(key, &self.key);
        let index =
            crate::map::hashing::get_index(&hashes, &self.displacements, self.entries.len())?;

        let Some(entry) = self.entries.get(index)? else {
            return Err(Error::new(crate::error::ErrorKind::IndexOutOfBounds { index, len: self.entries.len() }));
        };

        if entry.0.borrow() == key {
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}

/// A typed reference to a slice.
pub struct MapRef<K, V> {
    key: HashKey,
    entries: SliceRef<(K, V)>,
    displacements: SliceRef<(u32, u32)>,
}

impl<K, V> MapRef<K, V> {
    pub(crate) fn new(
        key: HashKey,
        entries: SliceRef<(K, V)>,
        displacements: SliceRef<(u32, u32)>,
    ) -> Self {
        Self {
            key,
            entries,
            displacements,
        }
    }
}

impl<'a, K: 'a, V: 'a> Bind<'a> for MapRef<K, V>
where
    K: Bind<'a>,
    V: Bind<'a>,
{
    type Output = Map<'a, K::Output, V::Output>;

    #[inline]
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
        Ok(Map {
            key: self.key,
            entries: self.entries.bind(buf)?,
            displacements: self.displacements.bind(buf)?,
        })
    }
}
