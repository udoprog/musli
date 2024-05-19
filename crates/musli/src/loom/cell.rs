#[cfg(loom)]
pub(crate) use loom::cell::UnsafeCell;

#[cfg(not(loom))]
pub(crate) struct UnsafeCell<T>
where
    T: ?Sized,
{
    inner: core::cell::UnsafeCell<T>,
}

#[cfg(not(loom))]
impl<T> UnsafeCell<T> {
    #[inline(always)]
    pub(crate) const fn new(inner: T) -> Self {
        Self {
            inner: core::cell::UnsafeCell::new(inner),
        }
    }
}

#[cfg(not(loom))]
impl<T> UnsafeCell<T>
where
    T: ?Sized,
{
    #[inline(always)]
    pub(crate) fn with_mut<O>(&self, f: impl FnOnce(*mut T) -> O) -> O {
        f(self.inner.get())
    }
}
