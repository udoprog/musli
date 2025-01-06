use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::needs_drop;
use core::ops::{Deref, DerefMut};

use super::{Alloc, AllocError, Allocator};

/// A Müsli-allocated pointer type that uniquely owns a heap allocation of type
/// `T`.
///
/// This is a [`Box`][std-box] type capable of using the allocator provided
/// through a [`Context`]. Therefore it can be safely used in no-std
/// environments.
///
/// [std-box]: std::boxed::Box
/// [`Context`]: crate::Context
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
    /// Allocates memory on the heap and then places `x` into it.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
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
    ///
    /// Zero-sized types:
    ///
    /// ```
    /// use musli::alloc::{AllocError, Box};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Box::new_in((), alloc)?;
    ///     assert_eq!(a.as_ref(), &());
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

impl<T, A> DerefMut for Box<T, A>
where
    A: Allocator,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The returned buffer is valid per construction.
        unsafe { &mut *self.buf.as_mut_ptr() }
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

impl<T, A> Drop for Box<T, A>
where
    A: Allocator,
{
    #[inline]
    fn drop(&mut self) {
        // SAFETY: Layout assumptions are correctly encoded in the type as it
        // was being allocated or grown.
        unsafe {
            if needs_drop::<T>() {
                self.buf.as_mut_ptr().drop_in_place();
            }
        }
    }
}

impl<T, A> fmt::Display for Box<T, A>
where
    T: fmt::Display,
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T, A> fmt::Debug for Box<T, A>
where
    T: fmt::Debug,
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, A> PartialEq for Box<T, A>
where
    T: PartialEq,
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[inline]
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T, A> PartialOrd for Box<T, A>
where
    T: PartialOrd,
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(&**self, &**other)
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T, A> Ord for Box<T, A>
where
    T: Ord,
    A: Allocator,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T, A> Eq for Box<T, A>
where
    T: Eq,
    A: Allocator,
{
}

impl<T, A> Hash for Box<T, A>
where
    T: Hash,
    A: Allocator,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T, A> Hasher for Box<T, A>
where
    T: Hasher,
    A: Allocator,
{
    #[inline]
    fn finish(&self) -> u64 {
        (**self).finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        (**self).write(bytes)
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        (**self).write_u8(i)
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        (**self).write_u16(i)
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        (**self).write_u32(i)
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        (**self).write_u64(i)
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        (**self).write_u128(i)
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        (**self).write_usize(i)
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        (**self).write_i8(i)
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        (**self).write_i16(i)
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        (**self).write_i32(i)
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        (**self).write_i64(i)
    }

    #[inline]
    fn write_i128(&mut self, i: i128) {
        (**self).write_i128(i)
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        (**self).write_isize(i)
    }
}
