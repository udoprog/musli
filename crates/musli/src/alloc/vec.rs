use core::cmp::Ordering;
use core::fmt;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;
#[cfg(feature = "alloc")]
use core::ptr::NonNull;
use core::slice;

#[cfg(feature = "alloc")]
use super::System;
use super::{AllocError, AllocSlice, Allocator};

/// A vector backed by an [`Allocator`].
pub struct Vec<T, A>
where
    A: Allocator,
{
    buf: A::AllocSlice<T>,
    len: usize,
}

#[cfg(feature = "alloc")]
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Vec<u8, crate::alloc::System>>();
};

impl<T, A> Vec<T, A>
where
    A: Allocator,
{
    /// Construct a buffer vector from raw parts.
    #[inline]
    const unsafe fn from_raw_parts(buf: A::AllocSlice<T>, len: usize) -> Self {
        Self { buf, len }
    }

    /// Access the raw allocation.
    #[cfg(test)]
    pub(crate) const fn raw(&self) -> &A::AllocSlice<T> {
        &self.buf
    }

    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     a.push(String::from("Hello"));
    ///     a.push(String::from("World"));
    ///
    ///     assert_eq!(a.as_slice(), ["Hello", "World"]);
    /// });
    /// ```
    #[inline]
    pub fn new_in(alloc: A) -> Self {
        Self {
            buf: alloc.alloc_slice::<T>(),
            len: 0,
        }
    }

    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::with_capacity_in(2, alloc)?;
    ///
    ///     a.push(String::from("Hello"))?;
    ///     a.push(String::from("World"))?;
    ///
    ///     assert_eq!(a.as_slice(), ["Hello", "World"]);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, AllocError> {
        let mut buf = alloc.alloc_slice::<T>();
        buf.resize(0, capacity)?;
        Ok(Self { buf, len: 0 })
    }

    /// Construct a new buffer vector.
    #[inline]
    pub const fn new(buf: A::AllocSlice<T>) -> Self {
        Self { buf, len: 0 }
    }

    /// Get the number of initialized elements in the buffer.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     assert_eq!(a.len(), 0);
    ///     a.extend_from_slice(b"Hello")?;
    ///     assert_eq!(a.len(), 5);
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     assert!(a.is_empty());
    ///     a.extend_from_slice(b"Hello")?;
    ///     assert!(!a.is_empty());
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Write a single item.
    ///
    /// Returns `true` if the item could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     a.push(b'H');
    ///     a.push(b'e');
    ///     a.push(b'l');
    ///     a.push(b'l');
    ///     a.push(b'o');
    ///
    ///     assert_eq!(a.as_slice(), b"Hello");
    /// });
    /// ```
    #[inline]
    pub fn push(&mut self, item: T) -> Result<(), AllocError> {
        if size_of::<T>() != 0 {
            self.buf.resize(self.len, 1)?;

            // SAFETY: The call to reserve ensures that we have enough capacity.
            unsafe {
                self.buf.as_mut_ptr().add(self.len).write(item);
            }
        }

        self.len += 1;
        Ok(())
    }

    /// Pop a single item from the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     a.push(String::from("foo"));
    ///     a.push(String::from("bar"));
    ///
    ///     assert_eq!(a.as_slice(), ["foo", "bar"]);
    ///
    ///     assert_eq!(a.pop().as_deref(), Some("bar"));
    ///     assert_eq!(a.pop().as_deref(), Some("foo"));
    ///     assert_eq!(a.pop(), None);
    /// });
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        // SAFETY: We know that the buffer is initialized up to `len`.
        unsafe { Some(ptr::read(self.buf.as_ptr().add(self.len))) }
    }

    /// Clear the buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     a.push(b'H');
    ///     a.push(b'e');
    ///     a.push(b'l');
    ///     a.push(b'l');
    ///     a.push(b'o');
    ///
    ///     assert_eq!(a.as_slice(), b"Hello");
    ///     a.clear();
    ///     assert_eq!(a.as_slice(), b"");
    /// });
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        // SAFETY: We know that the buffer is initialized up to `len`.
        unsafe { ptr::drop_in_place(slice::from_raw_parts_mut(self.buf.as_mut_ptr(), self.len)) }
        self.len = 0;
    }

    /// Get the initialized part of the buffer as a slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///     assert_eq!(a.as_slice(), b"");
    ///     a.extend_from_slice(b"Hello")?;
    ///     assert_eq!(a.as_slice(), b"Hello");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: We know that the buffer is initialized up to `self.len`.
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

    /// Get the initialized part of the buffer as a slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///     assert_eq!(a.as_slice(), b"");
    ///     a.extend_from_slice(b"Hello")?;
    ///     assert_eq!(a.as_slice(), b"Hello");
    ///     a.as_slice_mut().make_ascii_uppercase();
    ///     assert_eq!(a.as_slice(), b"HELLO");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: We know that the buffer is initialized up to `self.len`.
        unsafe { slice::from_raw_parts_mut(self.buf.as_mut_ptr(), self.len) }
    }

    #[inline]
    fn into_raw_parts(self) -> (A::AllocSlice<T>, usize) {
        let this = ManuallyDrop::new(self);

        // SAFETY: The interior buffer is valid and will not be dropped thanks to `ManuallyDrop`.
        unsafe {
            let buf = ptr::addr_of!(this.buf).read();
            (buf, this.len)
        }
    }
}

impl<T, A> Vec<T, A>
where
    A: Allocator,
    T: Copy,
{
    /// Write the given number of bytes.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///     assert_eq!(a.len(), 0);
    ///     a.extend_from_slice(b"Hello");
    ///     assert_eq!(a.len(), 5);
    /// });
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, items: &[T]) -> Result<(), AllocError> {
        if size_of::<T>() != 0 {
            self.buf.resize(self.len, items.len())?;

            // SAFETY: The call to reserve ensures that we have enough capacity.
            unsafe {
                self.buf
                    .as_mut_ptr()
                    .add(self.len)
                    .copy_from_nonoverlapping(items.as_ptr(), items.len());
            }
        }

        self.len += items.len();
        Ok(())
    }

    /// Write a buffer of the same type onto the current buffer.
    ///
    /// This allows allocators to provide more efficient means of extending the
    /// current buffer with one provided from the same allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///     let mut b = Vec::new_in(alloc);
    ///
    ///     a.extend_from_slice(b"Hello")?;
    ///     b.extend_from_slice(b" World")?;
    ///
    ///     a.extend(b)?;
    ///     assert_eq!(a.as_slice(), b"Hello World");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn extend(&mut self, other: Vec<T, A>) -> Result<(), AllocError> {
        let (other, other_len) = other.into_raw_parts();

        // Try to merge one buffer with another.
        if let Err(buf) = self.buf.try_merge(self.len, other, other_len) {
            let other = unsafe { Vec::<T, A>::from_raw_parts(buf, other_len) };
            return self.extend_from_slice(other.as_slice());
        }

        self.len += other_len;
        Ok(())
    }
}

/// Try to write a format string into the buffer.
///
/// ## Examples
///
/// ```
/// use core::fmt::Write;
///
/// use musli::alloc::Vec;
///
/// musli::alloc::default(|alloc| {
///     let mut a = Vec::new_in(alloc);
///     let world = "World";
///
///     write!(a, "Hello {world}")?;
///
///     assert_eq!(a.as_slice(), b"Hello World");
///     Ok(())
/// })?;
/// # Ok::<_, core::fmt::Error>(())
/// ```
impl<A> fmt::Write for Vec<u8, A>
where
    A: Allocator,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.extend_from_slice(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl<T, A> Deref for Vec<T, A>
where
    A: Allocator,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, A> DerefMut for Vec<T, A>
where
    A: Allocator,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T, A> fmt::Debug for Vec<T, A>
where
    T: fmt::Debug,
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T, A> Drop for Vec<T, A>
where
    A: Allocator,
{
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, A> PartialEq for Vec<T, A>
where
    T: PartialEq,
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T, A> Eq for Vec<T, A>
where
    T: Eq,
    A: Allocator,
{
}

impl<T, A> PartialOrd for Vec<T, A>
where
    T: PartialOrd,
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T, A> Ord for Vec<T, A>
where
    T: Ord,
    A: Allocator,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<T> From<rust_alloc::vec::Vec<T>> for Vec<T, System> {
    #[inline]
    fn from(value: rust_alloc::vec::Vec<T>) -> Self {
        // SAFETY: We know that the vector was allocated as expected using the
        // system allocator.
        unsafe {
            let mut value = ManuallyDrop::new(value);
            let ptr = NonNull::new_unchecked(value.as_mut_ptr());
            let len = value.len();
            let cap = value.capacity();

            let buf = System::slice_from_raw_parts(ptr, cap);

            Self { buf, len }
        }
    }
}
