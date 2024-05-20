#[cfg(loom)]
pub(crate) use loom::sync::atomic;

#[cfg(not(loom))]
pub(crate) use core::sync::atomic;

#[cfg(not(loom))]
#[inline(always)]
pub(crate) fn with_mut_usize<R>(
    ptr: &mut atomic::AtomicUsize,
    f: impl FnOnce(&mut usize) -> R,
) -> R {
    f(ptr.get_mut())
}

#[cfg(loom)]
#[inline(always)]
pub(crate) fn with_mut_usize<R>(
    ptr: &mut atomic::AtomicUsize,
    f: impl FnOnce(&mut usize) -> R,
) -> R {
    ptr.with_mut(f)
}
