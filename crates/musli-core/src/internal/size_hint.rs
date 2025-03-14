use crate::de::SizeHint;

// Uses the same heuristic as:
// https://github.com/serde-rs/serde/blob/d91f8ba950e2faf4db4e283e917ba2ee94a9b8a4/serde/src/de/size_hint.rs#L12
#[inline]
pub(crate) fn cautious<T>(hint: impl Into<SizeHint>) -> usize {
    const MAX_PREALLOC_BYTES: usize = 1024 * 1024;

    if size_of::<T>() == 0 {
        return 0;
    }

    hint.into()
        .or_default()
        .min(MAX_PREALLOC_BYTES / size_of::<T>())
}
