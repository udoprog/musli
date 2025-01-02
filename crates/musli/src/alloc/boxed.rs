use core::ops::Deref;

use super::{Alloc, AllocError, Allocator};

/// An allocation into the allocator `A`.
pub struct Box<T, A>
where
    A: Allocator,
{
    buf: A::Alloc<T>,
}

impl<T, A> Box<T, A>
where
    A: Allocator,
{
    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Box};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Box::new_in(10u32, alloc)?;
    ///     assert_eq!(a.as_ref(), &10u32);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn new_in(value: T, alloc: A) -> Result<Self, AllocError> {
        Ok(Self {
            buf: alloc.alloc(value)?,
        })
    }
}

unsafe impl<T, A> Send for Box<T, A>
where
    T: Send,
    A: Allocator,
{
}
unsafe impl<T, A> Sync for Box<T, A>
where
    T: Sync,
    A: Allocator,
{
}

impl<T, A> Deref for Box<T, A>
where
    A: Allocator,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: The returned buffer is valid per construction.
        unsafe { &*self.buf.as_ptr() }
    }
}

impl<T, A> AsRef<T> for Box<T, A>
where
    A: Allocator,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}
