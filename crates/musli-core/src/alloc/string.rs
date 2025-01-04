use core::cmp::Ordering;
use core::fmt;
use core::ops::Deref;
use core::str;
#[cfg(feature = "alloc")]
use rust_alloc::borrow::Cow;

#[cfg(feature = "alloc")]
use super::System;
use super::{AllocError, Allocator, Vec};

/// Collect a string into a string buffer.
#[inline]
pub(crate) fn collect_string<A, T>(alloc: A, value: T) -> Result<String<A>, AllocError>
where
    A: Allocator,
    T: fmt::Display,
{
    use core::fmt::Write;

    let mut string = String::new_in(alloc);

    if write!(string, "{value}").is_err() {
        return Err(AllocError);
    }

    Ok(string)
}

/// Wrapper around a buffer that is guaranteed to be a valid utf-8 string.
pub struct String<A>
where
    A: Allocator,
{
    buf: Vec<u8, A>,
}

const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<String<crate::alloc::Disabled>>();
};

impl<A> String<A>
where
    A: Allocator,
{
    /// Creates a new empty `String`.
    ///
    /// Given that the `String` is empty, this will not allocate any initial
    /// buffer. While that means that this initial operation is very
    /// inexpensive, it may cause excessive allocation later when you add data.
    /// If you have an idea of how much data the `String` will hold, consider
    /// the [`with_capacity_in`] method to prevent excessive re-allocation.
    ///
    /// [`with_capacity_in`]: String::with_capacity_in
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     _ = String::new_in(alloc);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn new_in(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    /// Extracts a string slice containing the entire `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut s = String::new_in(alloc);
    ///     s.push_str("foo")?;
    ///     assert_eq!(s.as_str(), "foo");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: Interactions ensure that data is valid utf-8.
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }

    /// Appends the given [`char`] to the end of this `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut s = String::new_in(alloc);
    ///
    ///     s.push_str("abc")?;
    ///
    ///     s.push('1')?;
    ///     s.push('2')?;
    ///     s.push('3')?;
    ///
    ///     assert_eq!("abc123", s);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn push(&mut self, c: char) -> Result<(), AllocError> {
        self.buf
            .extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes())
    }

    /// Appends a given string slice onto the end of this `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = String::new_in(alloc);
    ///
    ///     a.push_str("Hello")?;
    ///     a.push_str(" World")?;
    ///
    ///     assert_eq!(a.as_str(), "Hello World");
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), AllocError> {
        self.buf.extend_from_slice(s.as_bytes())
    }
}

impl<A> fmt::Write for String<A>
where
    A: Allocator,
{
    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c).map_err(|_| fmt::Error)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

impl<A> Deref for String<A>
where
    A: Allocator,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }
}

impl<A> fmt::Display for String<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A> fmt::Debug for String<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A> AsRef<str> for String<A>
where
    A: Allocator,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A> PartialEq for String<A>
where
    A: Allocator,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<A> Eq for String<A> where A: Allocator {}

impl<A> PartialOrd for String<A>
where
    A: Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for String<A>
where
    A: Allocator,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl From<rust_alloc::string::String> for String<System> {
    #[inline]
    fn from(value: rust_alloc::string::String) -> Self {
        Self {
            buf: Vec::from(value.into_bytes()),
        }
    }
}
macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b, A> PartialEq<$rhs> for $lhs
        where
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
        impl<'a, 'b, A> PartialEq<$lhs> for $rhs
        where
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

impl_eq! { String<A>, str }
impl_eq! { String<A>, &'a str }
#[cfg(feature = "alloc")]
impl_eq! { Cow<'a, str>, String<A> }
