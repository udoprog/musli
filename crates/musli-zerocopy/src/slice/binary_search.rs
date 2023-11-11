use core::cmp::Ordering;

use crate::buf::{Buf, Visit};
use crate::endian::ByteOrder;
use crate::error::Error;
use crate::pointer::{Ref, Size};
use crate::slice::Slice;
use crate::traits::ZeroCopy;

/// The result of a [`binary_search()`].
#[derive(Debug, PartialEq, Eq)]
pub enum BinarySearch {
    /// Found the element we were looking for during the search.
    Found(usize),
    /// Exact match could not be found, but this is the closest index where the
    /// searched for element could be inserted while still maintaining search
    /// order.
    Missing(usize),
}

/// Binary searches this slice for a given element. If the slice is not sorted,
/// the returned result is unspecified and meaningless.
///
/// If the value is found then [`BinarySearch::Found`] is returned, containing
/// the index of the matching element. If there are multiple matches, then any
/// one of the matches could be returned. The index is chosen deterministically,
/// but is subject to change in future versions of Rust. If the value is not
/// found then [`BinarySearch::Missing`] is returned, containing the index
/// where a matching element could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a uniquely
/// determined position; the second and third are not found; the fourth could
/// match any position in `[1, 4]`.
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::slice::{binary_search, BinarySearch};
///
/// let mut buf = OwnedBuf::new();
/// let slice = buf.store_slice(&[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
///
/// assert_eq!(binary_search(&buf, slice, &13)?, BinarySearch::Found(9));
/// assert_eq!(binary_search(&buf, slice, &4)?, BinarySearch::Missing(7));
/// assert_eq!(binary_search(&buf, slice, &100)?, BinarySearch::Missing(13));
///
/// let r = binary_search(&buf, slice, &1)?;
/// assert!(match r { BinarySearch::Found(1..=4) => true, _ => false, });
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn binary_search<T, E: ByteOrder, O: Size, Q>(
    buf: &Buf,
    slice: Ref<[T], E, O>,
    x: &Q,
) -> Result<BinarySearch, Error>
where
    T: ZeroCopy,
    T: Ord,
    Q: Visit<Target = T>,
{
    binary_search_by(buf, slice, |value| x.visit(buf, |x| value.cmp(x)))
}

/// Binary searches this slice with a comparator function.
///
/// The comparator function should return an order code that indicates whether
/// its argument is `Less`, `Equal` or `Greater` the desired target. If the
/// slice is not sorted or if the comparator function does not implement an
/// order consistent with the sort order of the underlying slice, the returned
/// result is unspecified and meaningless.
///
/// If the value is found then [`BinarySearch::Found`] is returned, containing
/// the index of the matching element. If there are multiple matches, then any
/// one of the matches could be returned. The index is chosen deterministically,
/// but is subject to change in future versions of Rust. If the value is not
/// found then [`BinarySearch::Missing`] is returned, containing the index where
/// a matching element could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use musli_zerocopy::OwnedBuf;
/// use musli_zerocopy::slice::{binary_search_by, BinarySearch};
///
/// let mut buf = OwnedBuf::new();
/// let slice = buf.store_slice(&[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
///
/// let s = 13;
/// assert_eq!(binary_search_by(&buf, slice, |v| Ok(v.cmp(&s)))?, BinarySearch::Found(9));
/// let s = 4;
/// assert_eq!(binary_search_by(&buf, slice, |v| Ok(v.cmp(&s)))?, BinarySearch::Missing(7));
/// let s = 100;
/// assert_eq!(binary_search_by(&buf, slice, |v| Ok(v.cmp(&s)))?, BinarySearch::Missing(13));
/// let s = 1;
/// let r = binary_search_by(&buf, slice, |v| Ok(v.cmp(&s)))?;
/// assert!(match r { BinarySearch::Found(1..=4) => true, _ => false, });
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub fn binary_search_by<S, T, F>(buf: &Buf, slice: S, mut f: F) -> Result<BinarySearch, Error>
where
    T: ZeroCopy,
    S: Slice<[T]>,
    F: FnMut(&T) -> Result<Ordering, Error>,
{
    // INVARIANTS:
    // - 0 <= left <= left + size = right <= slice.len()
    // - f returns Less for everything in slice[..left]
    // - f returns Greater for everything in slice[right..]
    let mut size = slice.len();
    let mut left = 0;
    let mut right = size;

    while left < right {
        let mid = left + size / 2;

        // The while condition means `size` is strictly positive, so `size/2
        // < size`. Thus `left + size/2 < left + size`, which coupled with
        // the `left + size <= slice.len()` invariant means we have `left +
        // size/2 < slice.len()`, and this is in-bounds.
        let value = buf.load(slice.get_unchecked(mid))?;
        let cmp = f(value)?;

        // The reason why we use if/else control flow rather than match
        // is because match reorders comparison operations, which is perf sensitive.
        // This is x86 asm for u8: https://rust.godbolt.org/z/8Y8Pra.
        if cmp == Ordering::Less {
            left = mid + 1;
        } else if cmp == Ordering::Greater {
            right = mid;
        } else {
            return Ok(BinarySearch::Found(mid));
        }

        size = right - left;
    }

    Ok(BinarySearch::Missing(left))
}
