use core::hash::Hash;

use crate::error::Error;
use crate::traits::{Read, Size, Write};
use crate::{MapRef, OwnedBuf, SliceBuilder};

/// A builder for a zero-copy map.
pub struct MapBuilder<K, V>
where
    K: Write,
    V: Write,
{
    entries: SliceBuilder<(K, V)>,
}

impl<K, V> MapBuilder<K, V>
where
    K: Size + Write,
    V: Size + Write,
{
    /// Construct a new slice builder.
    pub fn new() -> Self {
        Self {
            entries: SliceBuilder::new(),
        }
    }

    /// Push a value onto the slice.
    pub fn insert(&mut self, key: K, value: V) {
        self.entries.push((key, value));
    }

    /// Write a slice onto the slice builder.
    pub fn build(mut self, buf: &mut OwnedBuf) -> Result<MapRef<K, V>, Error>
    where
        K: Hash + for<'u> Read<'u>,
        V: for<'u> Read<'u>,
    {
        let mut hash_state = crate::map::generator::generate_hash(self.entries.as_slice())?;

        for a in 0..hash_state.map.len() {
            loop {
                let b = hash_state.map[a];

                if hash_state.map[a] == a {
                    break;
                }

                self.entries.swap(a, b);
                hash_state.map.swap(a, b);
            }
        }

        let entries = self.entries.build(buf);

        let mut displacements = SliceBuilder::new();

        for d in hash_state.displacements {
            displacements.push(d);
        }

        let displacements = displacements.build(buf);
        Ok(MapRef::new(hash_state.key, entries, displacements))
    }
}
