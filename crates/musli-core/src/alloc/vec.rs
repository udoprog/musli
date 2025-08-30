use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use crate::de::{DecodeBytes, UnsizedVisitor};
use crate::{Context, Decoder};

#[cfg(feature = "alloc")]
use super::GlobalAllocator;
use super::{Alloc, AllocError, Allocator};

/// A Müsli-allocated contiguous growable array type, written as `Vec<T>`, short
/// for 'vector'.
///
/// This is a [`Vec`][alloc-vec] style type capable of using the [`Allocator`]
/// provided through a [`Context`]. Therefore it can be safely used in no-alloc
/// environments.
///
/// [alloc-vec]: rust_alloc::vec::Vec
pub struct Vec<T, A>
where
    A: Allocator,
{
    buf: A::Alloc<T>,
    len: usize,
}

impl<T, A> Vec<T, A>
where
    A: Allocator,
{
    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
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
    pub fn new_in(alloc: A) -> Self {
        Self {
            buf: alloc.alloc_empty::<T>(),
            len: 0,
        }
    }

    /// Coerce into a std vector.
    #[cfg(feature = "alloc")]
    pub fn into_std(self) -> Result<rust_alloc::vec::Vec<T>, Self> {
        if !A::IS_GLOBAL {
            return Err(self);
        }

        let mut this = ManuallyDrop::new(self);

        // SAFETY: The implementation requirements of `Allocator` requires that
        // this is possible.
        unsafe {
            let ptr = this.buf.as_mut_ptr();
            let cap = this.buf.capacity();

            Ok(rust_alloc::vec::Vec::from_raw_parts(ptr, this.len, cap))
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
        let mut buf = alloc.alloc_empty::<T>();
        buf.resize(0, capacity)?;
        Ok(Self { buf, len: 0 })
    }

    /// Returns the number of elements in the vector, also referred to as its
    /// 'length'.
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

    /// Returns the total number of elements the vector can hold without
    /// reallocating.
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
    ///     assert_eq!(a.capacity(), 0);
    ///
    ///     a.extend_from_slice(b"Hello")?;
    ///     assert_eq!(a.len(), 5);
    ///     assert!(a.capacity() >= 5);
    ///
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `Vec<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut vec = Vec::new_in(alloc);
    ///     vec.push(1)?;
    ///     vec.reserve(10)?;
    ///     assert!(vec.capacity() >= 11);
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    pub fn reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        if size_of::<T>() != 0 {
            self.buf.resize(self.len, additional)?;
        }

        Ok(())
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
    ///     a.as_mut_slice().make_ascii_uppercase();
    ///     assert_eq!(a.as_slice(), b"HELLO");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: We know that the buffer is initialized up to `self.len`.
        unsafe { slice::from_raw_parts_mut(self.buf.as_mut_ptr(), self.len) }
    }

    /// Deconstruct a vector into its raw parts.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{Allocator, AllocError, Vec};
    ///
    /// fn operate<A>(alloc: A) -> Result<(), AllocError>
    /// where
    ///     A: Allocator
    /// {
    ///     let mut a = Vec::new_in(alloc);
    ///     a.extend_from_slice(b"abc")?;
    ///     let (buf, len) = a.into_raw_parts();
    ///
    ///     let b = Vec::<_, A>::from_raw_parts(buf, len);
    ///     assert_eq!(b.as_slice(), b"abc");
    ///     Ok::<_, AllocError>(())
    /// }
    ///
    /// musli::alloc::default(|alloc| operate(alloc))?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn into_raw_parts(self) -> (A::Alloc<T>, usize) {
        let this = ManuallyDrop::new(self);

        // SAFETY: The interior buffer is valid and will not be dropped thanks to `ManuallyDrop`.
        unsafe {
            let buf = ptr::addr_of!(this.buf).read();
            (buf, this.len)
        }
    }

    /// Construct a vector from raw parts.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{Allocator, AllocError, Vec};
    ///
    /// fn operate<A>(alloc: A) -> Result<(), AllocError>
    /// where
    ///     A: Allocator
    /// {
    ///     let mut a = Vec::new_in(alloc);
    ///     a.extend_from_slice(b"abc")?;
    ///     let (buf, len) = a.into_raw_parts();
    ///
    ///     let b = Vec::<_, A>::from_raw_parts(buf, len);
    ///     assert_eq!(b.as_slice(), b"abc");
    ///     Ok::<_, AllocError>(())
    /// }
    ///
    /// musli::alloc::default(|alloc| operate(alloc))?;
    /// # Ok::<_, musli::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn from_raw_parts(buf: A::Alloc<T>, len: usize) -> Self {
        Self { buf, len }
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal
    /// invariants of the type. Normally changing the length of a vector is done
    /// using one of the safe operations instead, such as [`extend`], or
    /// [`clear`].
    ///
    /// [`extend`]: Extend::extend
    /// [`clear`]: Vec::clear
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to [`capacity()`].
    /// - The elements at `old_len..new_len` must be initialized.
    ///
    /// [`capacity()`]: Vec::capacity
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    /// Access a reference to the raw underlying allocation.
    pub const fn raw(&self) -> &A::Alloc<T> {
        &self.buf
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
            let other = Vec::<T, A>::from_raw_parts(buf, other_len);
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
        self.as_mut_slice()
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

impl<T, A> AsRef<[T]> for Vec<T, A>
where
    A: Allocator,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, A> AsMut<[T]> for Vec<T, A>
where
    A: Allocator,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b, T, A> PartialEq<$rhs> for $lhs
        where
            T: PartialEq,
            A: Allocator,
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &$rhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b, T, A> PartialEq<$lhs> for $rhs
        where
            T: PartialEq,
            A: Allocator,
        {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &$lhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }
    };
}

macro_rules! impl_eq_array {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b, T, A, const N: usize> PartialEq<$rhs> for $lhs
        where
            T: PartialEq,
            A: Allocator,
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &$rhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b, T, A, const N: usize> PartialEq<$lhs> for $rhs
        where
            T: PartialEq,
            A: Allocator,
        {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &$lhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { Vec<T, A>, [T] }
impl_eq! { Vec<T, A>, &'a [T] }
impl_eq_array! { Vec<T, A>, [T; N] }
impl_eq_array! { Vec<T, A>, &'a [T; N] }

impl<T, A, B> PartialEq<Vec<T, B>> for Vec<T, A>
where
    T: PartialEq,
    A: Allocator,
    B: Allocator,
{
    #[inline]
    fn eq(&self, other: &Vec<T, B>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T, A> Eq for Vec<T, A>
where
    T: Eq,
    A: Allocator,
{
}

impl<T, A, B> PartialOrd<Vec<T, B>> for Vec<T, A>
where
    T: PartialOrd,
    A: Allocator,
    B: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Vec<T, B>) -> Option<Ordering> {
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

impl<T, A> Borrow<[T]> for Vec<T, A>
where
    A: Allocator,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self
    }
}

/// Conversion from a std [`Vec`][std-vec] to a Müsli-allocated [`Vec`] in the
/// [`GlobalAllocator`] allocator.
///
/// [std-vec]: rust_alloc::vec::Vec
///
/// # Examples
///
/// ```
/// use musli::alloc::{Vec, Global};
///
/// let values = vec![1, 2, 3, 4];
/// let values2 = Vec::<_, Global>::from(values);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<T, A> From<rust_alloc::vec::Vec<T>> for Vec<T, A>
where
    A: GlobalAllocator,
{
    #[inline]
    fn from(value: rust_alloc::vec::Vec<T>) -> Self {
        use core::ptr::NonNull;

        // SAFETY: We know that the vector was allocated as expected using the
        // global allocator.
        unsafe {
            let mut value = ManuallyDrop::new(value);
            let ptr = NonNull::new_unchecked(value.as_mut_ptr());
            let len = value.len();
            let cap = value.capacity();

            let buf = A::slice_from_raw_parts(ptr, cap);
            Vec::from_raw_parts(buf, len)
        }
    }
}

/// Decode implementation for a Müsli-allocated byte array stored in a [`Vec`].
///
/// # Examples
///
/// ```
/// use musli::alloc::Vec;
/// use musli::{Allocator, Decode};
///
/// #[derive(Decode)]
/// struct Struct<A> where A: Allocator {
///     #[musli(bytes)]
///     field: Vec<u8, A>
/// }
/// ```
impl<'de, M, A> DecodeBytes<'de, M, A> for Vec<u8, A>
where
    A: Allocator,
{
    const DECODE_BYTES_PACKED: bool = false;

    #[inline]
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        struct Visitor;

        #[crate::trait_defaults(crate)]
        impl<C> UnsizedVisitor<'_, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = Vec<u8, Self::Allocator>;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_owned(
                self,
                _: C,
                value: Vec<u8, Self::Allocator>,
            ) -> Result<Self::Ok, Self::Error> {
                Ok(value)
            }

            #[inline]
            fn visit_ref(self, cx: C, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                let mut buf = Vec::new_in(cx.alloc());
                buf.extend_from_slice(bytes).map_err(cx.map())?;
                Ok(buf)
            }
        }

        decoder.decode_bytes(Visitor)
    }
}

crate::internal::macros::slice_sequence! {
    cx,
    Vec<T, A>,
    || Vec::new_in(cx.alloc()),
    |vec, value| vec.push(value).map_err(cx.map())?,
    |vec, capacity| vec.reserve(capacity).map_err(cx.map())?,
    |capacity| Vec::with_capacity_in(capacity, cx.alloc()).map_err(cx.map())?,
}
