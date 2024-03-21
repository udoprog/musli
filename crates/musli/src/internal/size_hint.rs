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
    match SizeHint::from(hint) {
        SizeHint::Any => 0,
        SizeHint::Exact(n) => n.min(4096),
    }
}
