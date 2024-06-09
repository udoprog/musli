use core::fmt::{self, Arguments};
use core::mem::ManuallyDrop;
#[cfg(test)]
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use crate::buf::{Buf, Error};
use crate::Allocator;

/// A vector backed by an [`Allocator`] [`Buf`].
pub struct BufVec<B>
where
    B: Buf,
{
    buf: B,
    len: usize,
}

impl<B> BufVec<B>
where
    B: Buf,
{
    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///
    ///     a.push(String::from("Hello"));
    ///     a.push(String::from("World"));
    ///
    ///     assert_eq!(a.as_slice(), ["Hello", "World"]);
    /// });
    /// ```
    pub fn new_in<'a, T>(alloc: &'a (impl ?Sized + Allocator<Buf<'a, T> = B>)) -> Self
    where
        T: 'a,
    {
        Self {
            buf: alloc.alloc::<T>(),
            len: 0,
        }
    }

    /// Construct a new buffer vector.
    #[inline]
    pub const fn new(buf: B) -> Self {
        Self { buf, len: 0 }
    }

    /// Get the number of initialized elements in the buffer.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///
    ///     assert_eq!(a.len(), 0);
    ///     a.write(b"Hello");
    ///     assert_eq!(a.len(), 5);
    /// });
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///
    ///     assert!(a.is_empty());
    ///     a.write(b"Hello");
    ///     assert!(!a.is_empty());
    /// });
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
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
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
    pub fn push(&mut self, item: B::Item) -> bool {
        if !self.buf.resize(self.len, 1) {
            return false;
        }

        // SAFETY: The call to reserve ensures that we have enough capacity.
        unsafe {
            self.buf.as_mut_ptr().add(self.len).write(item);
            self.len += 1;
        }

        true
    }

    /// Pop a single item from the buffer.
    ///
    /// Returns `None` if the buffer is empty.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
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
    pub fn pop(&mut self) -> Option<B::Item> {
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
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
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
    pub fn clear(&mut self) {
        if (self.buf.as_mut_ptr() as usize) % std::mem::align_of::<B::Item>() != 0 {
            std::println!("{}: {}", self.len, std::any::type_name::<B::Item>());
            panic!("Unaligned pointer: {:p}", self.buf.as_mut_ptr());
        }

        // SAFETY: We know that the buffer is initialized up to `len`.
        unsafe { ptr::drop_in_place(slice::from_raw_parts_mut(self.buf.as_mut_ptr(), self.len)) }
        self.len = 0;
    }

    /// Get the initialized part of the buffer as a slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///     assert_eq!(a.as_slice(), b"");
    ///     a.write(b"Hello");
    ///     assert_eq!(a.as_slice(), b"Hello");
    /// });
    /// ```
    pub fn as_slice(&self) -> &[B::Item] {
        // SAFETY: We know that the buffer is initialized up to `self.len`.
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

    #[inline]
    fn into_parts(self) -> (B, usize) {
        let this = ManuallyDrop::new(self);

        // SAFETY: The interior buffer is valid and will not be dropped thanks to `ManuallyDrop`.
        unsafe {
            let buf = ptr::addr_of!(this.buf).read();
            (buf, this.len)
        }
    }
}

impl<B> BufVec<B>
where
    B: Buf,
    B::Item: Copy,
{
    /// Write the given number of bytes.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///     assert_eq!(a.len(), 0);
    ///     a.write(b"Hello");
    ///     assert_eq!(a.len(), 5);
    /// });
    /// ```
    pub fn write(&mut self, items: &[B::Item]) -> bool {
        if !self.buf.resize(self.len, items.len()) {
            return false;
        }

        // SAFETY: The call to reserve ensures that we have enough capacity.
        unsafe {
            self.buf
                .as_mut_ptr()
                .add(self.len)
                .copy_from_nonoverlapping(items.as_ptr(), items.len());
            self.len += items.len();
        }

        true
    }
}

impl<B> BufVec<B>
where
    B: Buf<Item = u8>,
{
    /// Write a buffer of the same type onto the current buffer.
    ///
    /// This allows allocators to provide more efficient means of extending the
    /// current buffer with one provided from the same allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///     let mut b = BufVec::new_in(alloc).expect("allocation failed");
    ///
    ///     a.write(b"Hello");
    ///     b.write(b" World");
    ///
    ///     a.extend(b);
    ///     assert_eq!(a.as_slice(), b"Hello World");
    /// });
    /// ```
    #[inline]
    pub fn extend<U>(&mut self, other: BufVec<U>) -> bool
    where
        U: Buf<Item = u8>,
    {
        let (other, other_len) = other.into_parts();

        // Try to merge one buffer with another.
        if let Err(buf) = self.buf.try_merge(self.len, other, other_len) {
            let other = BufVec {
                buf,
                len: other_len,
            };

            return self.write(other.as_slice());
        }

        self.len += other_len;
        true
    }

    /// Try to write a format string into the buffer.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::{Allocator, Buf};
    /// use musli::buf::BufVec;
    ///
    /// musli::allocator::default!(|alloc| {
    ///     let mut a = BufVec::new_in(alloc).expect("allocation failed");
    ///     let world = "World";
    ///
    ///     write!(a, "Hello {world}")?;
    ///
    ///     assert_eq!(a.as_slice(), b"Hello World");
    /// });
    /// # Ok::<(), musli::buf::Error>(())
    /// ```
    #[inline]
    pub fn write_fmt(&mut self, arguments: Arguments<'_>) -> Result<(), Error> {
        struct Write<'a, B>(&'a mut BufVec<B>)
        where
            B: Buf;

        impl<B> fmt::Write for Write<'_, B>
        where
            B: Buf<Item = u8>,
        {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                if !self.0.write(s.as_bytes()) {
                    return Err(fmt::Error);
                }

                Ok(())
            }
        }

        let mut write = Write(self);
        fmt::write(&mut write, arguments).map_err(|_| Error)
    }
}

#[cfg(test)]
impl<B> Deref for BufVec<B>
where
    B: Buf,
{
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

#[cfg(test)]
impl<B> DerefMut for BufVec<B>
where
    B: Buf,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<B> Drop for BufVec<B>
where
    B: Buf,
{
    fn drop(&mut self) {
        self.clear();
    }
}
