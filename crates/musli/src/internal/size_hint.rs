#[cfg(any(feature = "std", feature = "alloc"))]
#[inline]
pub(crate) fn cautious(hint: Option<usize>) -> usize {
    hint.unwrap_or(0).min(4096)
}
