#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
use crate::de::SizeHint;

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub(crate) fn cautious<S>(hint: S) -> usize
where
    SizeHint: From<S>,
{
    SizeHint::from(hint).or_default().min(4096)
}
