use core::cell::UnsafeCell;

/// An option stored in a static location.
///
/// This interior mutability ensures that the static value is *not* stored in
/// a read-only location per the Rust reference.
pub struct Opt<T> {
    inner: UnsafeCell<T>,
}

impl<T> Opt<T> {
    /// Construct a new option.
    #[inline]
    #[doc(hidden)]
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    /// Replace the inner value.
    ///
    /// # Safety
    ///
    /// Caller must ensure nobody is accessing the stored value.
    #[inline]
    #[doc(hidden)]
    pub unsafe fn replace(&self, value: T) -> T {
        unsafe { self.inner.get().replace(value) }
    }

    /// Get the value.
    #[inline]
    #[doc(hidden)]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        // Get a clone of the inner value.
        unsafe { (*self.inner.get()).clone() }
    }
}

unsafe impl<T> Sync for Opt<T> where T: Sync {}
