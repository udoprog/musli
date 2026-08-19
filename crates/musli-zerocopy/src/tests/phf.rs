use alloc::vec::Vec;

use crate::{OwnedBuf, phf};

/// Every key stored in a perfect hash map must be found again.
///
/// The entries are permuted into the order dictated by the generated hash, and
/// getting the direction of that permutation wrong only shows up once the
/// permutation contains a cycle longer than a transposition.
#[test]
fn map_lookup_after_permutation() {
    // Keys which produce a three element cycle.
    let entries = [(1u32, 10u32), (41, 20), (37, 30)];

    let mut buf = OwnedBuf::new();
    let map = phf::store_map(&mut buf, entries).unwrap();
    let map = buf.bind(map).unwrap();

    for (key, value) in entries {
        assert_eq!(map.get(&key).unwrap(), Some(&value), "key {key}");
    }
}

/// Exhaustively check that every key in a generated map can be looked up again.
#[test]
fn map_lookup_is_exhaustive() {
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    let mut rng = Rng(0x1234_5678);

    for round in 0..500u64 {
        let count = (round % 32) as usize;
        let mut keys = Vec::new();

        for _ in 0..count {
            let key = (rng.next() % 200) as u32;

            if !keys.contains(&key) {
                keys.push(key);
            }
        }

        let entries = keys
            .iter()
            .map(|&k| (k, k.wrapping_mul(7)))
            .collect::<Vec<_>>();

        let mut buf = OwnedBuf::new();
        let map = phf::store_map(&mut buf, entries).unwrap();
        let map = buf.bind(map).unwrap();

        for &key in &keys {
            assert_eq!(
                map.get(&key).unwrap(),
                Some(&key.wrapping_mul(7)),
                "round {round}, key {key}"
            );
        }
    }
}

/// The same applies to sets.
#[test]
fn set_lookup_is_exhaustive() {
    let keys = [1u32, 41, 37];

    let mut buf = OwnedBuf::new();
    let set = phf::store_set(&mut buf, keys).unwrap();
    let set = buf.bind(set).unwrap();

    for key in keys {
        assert!(set.contains(&key).unwrap(), "key {key}");
    }
}
