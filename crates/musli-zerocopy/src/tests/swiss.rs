use alloc::vec::Vec;

use crate::{OwnedBuf, swiss};

/// The reported length of a map must be the number of entries it holds.
///
/// The table reserves a portion of its buckets to stay empty, so the number of
/// buckets which have been filled cannot be derived from the bucket mask alone.
#[test]
fn map_len_is_the_number_of_entries() {
    for n in [
        0usize, 1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 64, 129, 500,
    ] {
        let entries = (0..n as u32).map(|i| (i, i * 2)).collect::<Vec<_>>();

        let mut buf = OwnedBuf::new();
        let map = swiss::store_map(&mut buf, entries).unwrap();

        assert_eq!(map.len(), n, "unbound map of {n} entries");
        assert_eq!(map.is_empty(), n == 0, "unbound map of {n} entries");

        let map = buf.bind(map).unwrap();

        assert_eq!(map.len(), n, "map of {n} entries");
        assert_eq!(map.is_empty(), n == 0, "map of {n} entries");

        for i in 0..n as u32 {
            assert_eq!(map.get(&i).unwrap(), Some(&(i * 2)), "map of {n}, key {i}");
        }
    }
}

/// The same applies to sets.
#[test]
fn set_len_is_the_number_of_entries() {
    for n in [
        0usize, 1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 64, 129, 500,
    ] {
        let entries = (0..n as u32).collect::<Vec<_>>();

        let mut buf = OwnedBuf::new();
        let set = swiss::store_set(&mut buf, entries).unwrap();
        let set = buf.bind(set).unwrap();

        for i in 0..n as u32 {
            assert!(set.contains(&i).unwrap(), "set of {n}, key {i}");
        }
    }
}
