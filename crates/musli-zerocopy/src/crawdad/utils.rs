use core::cmp::Ordering;

use alloc::vec::Vec;

/// Returns the smallest number of bytes that can encode `n`.
#[inline(always)]
pub const fn pack_size(n: u32) -> u8 {
    if n < 1 << 8 {
        1
    } else if n < 1 << 16 {
        2
    } else if n < 1 << 24 {
        3
    } else {
        4
    }
}

/// Pushes the lowest `nbytes` bytes of `n` to `vec`.
#[inline(always)]
pub fn pack_u32(vec: &mut Vec<u8>, n: u32, nbytes: u8) {
    vec.extend_from_slice(&n.to_le_bytes()[..usize::from(nbytes)]);
}

/// Extracts the head `nbytes` bytes of `slice`.
#[inline(always)]
pub fn unpack_u32(slice: &[u8], nbytes: u8) -> u32 {
    let mut n_array = [0; 4];
    n_array[..usize::from(nbytes)].copy_from_slice(&slice[..usize::from(nbytes)]);
    u32::from_le_bytes(n_array)
}

/// Returns `(lcp, ord)` such that
///  - lcp: Length of longest commom prefix of `a` and `b`.
///  - ord: `Ordering` between `a` and `b`.
#[inline(always)]
pub fn longest_common_prefix(a: &[char], b: &[char]) -> (usize, Ordering) {
    let min_len = a.len().min(b.len());
    for i in 0..min_len {
        if a[i] != b[i] {
            return (i, a[i].cmp(&b[i]));
        }
    }
    (min_len, a.len().cmp(&b.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_common_prefix() {
        assert_eq!(
            longest_common_prefix(&['a', 'b'], &['a', 'b', 'c']),
            (2, Ordering::Less)
        );
        assert_eq!(
            longest_common_prefix(&['a', 'b'], &['a', 'b']),
            (2, Ordering::Equal)
        );
        assert_eq!(
            longest_common_prefix(&['a', 'b', 'c'], &['a', 'b']),
            (2, Ordering::Greater)
        );
    }
}
