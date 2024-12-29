use crate::de::SizeHint;

#[inline]
pub(crate) fn cautious<S>(hint: S) -> usize
where
    SizeHint: From<S>,
{
    SizeHint::from(hint).or_default().min(4096)
}
