use core::borrow::Borrow;
use core::cmp::Ordering;
use core::error::Error;
use core::fmt;
use core::ops::Deref;
use core::str::{self, Utf8Error};

#[cfg(feature = "alloc")]
use rust_alloc::borrow::Cow;

use crate::de::UnsizedVisitor;
use crate::{Context, Decode, Decoder, Encode, Encoder};

use super::{AllocError, Allocator, GlobalAllocator, Vec};

/// Collect a string into a string buffer.
#[inline]
pub(crate) fn collect_string<A>(alloc: A, value: impl fmt::Display) -> Result<String<A>, AllocError>
where
    A: Allocator,
{
    use core::fmt::Write;

    let mut string = String::new_in(alloc);

    if write!(string, "{value}").is_err() {
        return Err(AllocError);
    }

    Ok(string)
}

/// A Müsli-allocated UTF-8–encoded, growable string.
///
/// `String` is the most common string type. It has ownership over the contents
/// of the string, stored in a allocated buffer. It is closely related to its
/// borrowed counterpart, the primitive [`str`].
///
/// This is a [`String`][alloc-string] style type capable of using the
/// [`Allocator`] provided through a [`Context`]. Therefore it can be safely
/// used in no-alloc environments.
///
/// [alloc-string]: rust_alloc::string::String
/// [`str`]: prim@str
pub struct String<A>
where
    A: Allocator,
{
    vec: Vec<u8, A>,
}

/// A possible error value when converting a `String` from a UTF-8 byte vector.
///
/// This type is the error type for the [`from_utf8`] method on [`String`]. It
/// is designed in such a way to carefully avoid reallocations: the
/// [`into_bytes`] method will give back the byte vector that was used in the
/// conversion attempt.
///
/// [`from_utf8`]: String::from_utf8
/// [`into_bytes`]: FromUtf8Error::into_bytes
///
/// The [`Utf8Error`] type provided by [`std::str`] represents an error that may
/// occur when converting a slice of [`u8`]s to a [`&str`]. In this sense, it's
/// an analogue to `FromUtf8Error`, and you can get one from a `FromUtf8Error`
/// through the [`utf8_error`] method.
///
/// [`Utf8Error`]: str::Utf8Error "std::str::Utf8Error"
/// [`std::str`]: core::str "std::str"
/// [`&str`]: prim@str "&str"
/// [`utf8_error`]: FromUtf8Error::utf8_error
///
/// # Examples
///
/// ```
/// // some invalid bytes, in a vector
/// let bytes = vec![0, 159];
///
/// let value = String::from_utf8(bytes);
///
/// assert!(value.is_err());
/// assert_eq!(vec![0, 159], value.unwrap_err().into_bytes());
/// ```
pub struct FromUtf8Error<A>
where
    A: Allocator,
{
    bytes: Vec<u8, A>,
    error: Utf8Error,
}

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
            vec: Vec::new_in(alloc),
        }
    }

    /// Creates a new empty `String` with at least the specified capacity.
    ///
    /// `String`s have an internal buffer to hold their data. The capacity is
    /// the length of that buffer, and can be queried with the [`capacity`]
    /// method. This method creates an empty `String`, but one with an initial
    /// buffer that can hold at least `capacity` bytes. This is useful when you
    /// may be appending a bunch of data to the `String`, reducing the number of
    /// reallocations it needs to do.
    ///
    /// [`capacity`]: String::capacity
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new_in`] method.
    ///
    /// [`new_in`]: String::new_in
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut s = String::with_capacity_in(10, alloc)?;
    ///
    ///     // The String contains no chars, even though it has capacity for more
    ///     assert_eq!(s.len(), 0);
    ///
    ///     // These are all done without reallocating...
    ///     let cap = s.capacity();
    ///
    ///     for _ in 0..10 {
    ///         s.push('a')?;
    ///     }
    ///
    ///     assert_eq!(s.capacity(), cap);
    ///
    ///     // ...but this may make the string reallocate
    ///     s.push('a')?;
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, AllocError> {
        Ok(Self {
            vec: Vec::with_capacity_in(capacity, alloc)?,
        })
    }

    /// Coerce into a std string.
    #[cfg(feature = "alloc")]
    pub fn into_std(self) -> Result<rust_alloc::string::String, Self> {
        match self.vec.into_std() {
            Ok(vec) => {
                // SAFETY: The buffer is guaranteed to be valid utf-8.
                unsafe { Ok(rust_alloc::string::String::from_utf8_unchecked(vec)) }
            }
            Err(vec) => Err(Self { vec }),
        }
    }

    /// Converts a vector of bytes to a `String`.
    ///
    /// A string ([`String`]) is made of bytes ([`u8`]), and a vector of bytes
    /// ([`Vec<u8>`]) is made of bytes, so this function converts between the
    /// two. Not all byte slices are valid `String`s, however: `String` requires
    /// that it is valid UTF-8. `from_utf8()` checks to ensure that the bytes
    /// are valid UTF-8, and then does the conversion.
    ///
    /// If you are sure that the byte slice is valid UTF-8, and you don't want
    /// to incur the overhead of the validity check, there is an unsafe version
    /// of this function, [`from_utf8_unchecked`], which has the same behavior
    /// but skips the check.
    ///
    /// This method will take care to not copy the vector, for efficiency's
    /// sake.
    ///
    /// If you need a [`&str`] instead of a `String`, consider
    /// [`str::from_utf8`].
    ///
    /// The inverse of this method is [`into_bytes`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not UTF-8 with a description as to why
    /// the provided bytes are not UTF-8. The vector you moved in is also
    /// included.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut sparkle_heart = Vec::<u8, _>::new_in(alloc);
    ///     // some bytes, in a vector
    ///     sparkle_heart.extend_from_slice(&[240, 159, 146, 150])?;
    ///
    ///     let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
    ///
    ///     assert_eq!("💖", sparkle_heart);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    ///
    /// Incorrect bytes:
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut sparkle_heart = Vec::<u8, _>::new_in(alloc);
    ///     // some bytes, in a vector
    ///     sparkle_heart.extend_from_slice(&[0, 159, 146, 150])?;
    ///
    ///     assert!(String::from_utf8(sparkle_heart).is_err());
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    ///
    /// See the docs for [`FromUtf8Error`] for more details on what you can do
    /// with this error.
    ///
    /// [`from_utf8_unchecked`]: String::from_utf8_unchecked
    /// [`Vec<u8>`]: crate::alloc::Vec "Vec"
    /// [`&str`]: prim@str "&str"
    /// [`into_bytes`]: String::into_bytes
    #[inline]
    pub fn from_utf8(vec: Vec<u8, A>) -> Result<String<A>, FromUtf8Error<A>> {
        match str::from_utf8(&vec) {
            Ok(..) => Ok(String { vec }),
            Err(e) => Err(FromUtf8Error {
                bytes: vec,
                error: e,
            }),
        }
    }

    /// Converts a vector of bytes to a `String` without checking that the
    /// string contains valid UTF-8.
    ///
    /// See the safe version, [`from_utf8`], for more details.
    ///
    /// [`from_utf8`]: String::from_utf8
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid UTF-8. If this constraint is violated, it may cause
    /// memory unsafety issues with future users of the `String`, as the rest of
    /// the standard library assumes that `String`s are valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut sparkle_heart = Vec::<u8, _>::new_in(alloc);
    ///     // some bytes, in a vector
    ///     sparkle_heart.extend_from_slice(&[240, 159, 146, 150])?;
    ///
    ///     let sparkle_heart = unsafe {
    ///         String::from_utf8_unchecked(sparkle_heart)
    ///     };
    ///
    ///     assert_eq!("💖", sparkle_heart);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(vec: Vec<u8, A>) -> String<A> {
        String { vec }
    }

    /// Converts a `String` into a byte vector.
    ///
    /// This consumes the `String`, so we do not need to copy its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut s = String::new_in(alloc);
    ///     s.push_str("hello")?;
    ///     let bytes = s.into_bytes();
    ///
    ///     assert_eq!(&[104, 101, 108, 108, 111][..], &bytes[..]);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8, A> {
        self.vec
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
        unsafe { str::from_utf8_unchecked(self.vec.as_slice()) }
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
        self.vec
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
        self.vec.extend_from_slice(s.as_bytes())
    }

    /// Returns this `String`'s capacity, in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let s = String::with_capacity_in(10, alloc)?;
    ///
    ///     assert!(s.capacity() >= 10);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Returns the length of this `String`, in bytes, not [`char`]s or
    /// graphemes. In other words, it might not be what a human considers the
    /// length of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = String::new_in(alloc);
    ///     a.push_str("foo")?;
    ///     assert_eq!(a.len(), 3);
    ///
    ///     let mut fancy_f = String::new_in(alloc);
    ///     fancy_f.push_str("ƒoo")?;
    ///     assert_eq!(fancy_f.len(), 4);
    ///     assert_eq!(fancy_f.chars().count(), 3);
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if this `String` has a length of zero, and `false`
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut v = String::new_in(alloc);
    ///     assert!(v.is_empty());
    ///
    ///     v.push('a')?;
    ///     assert!(!v.is_empty());
    ///     Ok::<_, AllocError>(())
    /// })?;
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<A> Clone for String<A>
where
    A: GlobalAllocator,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
        }
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
        unsafe { str::from_utf8_unchecked(self.vec.as_slice()) }
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

impl<A> Borrow<str> for String<A>
where
    A: Allocator,
{
    #[inline]
    fn borrow(&self) -> &str {
        self
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

/// Conversion from a std [`String`][std-string] to a Müsli-allocated [`String`]
/// in a [`GlobalAllocator`] allocator.
///
/// [std-string]: rust_alloc::string::String
///
/// # Examples
///
/// ```
/// use musli::alloc::{String, Global};
///
/// let value = std::string::String::from("Hello World");
/// let value2 = String::<Global>::from(value);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<A> From<rust_alloc::string::String> for String<A>
where
    A: GlobalAllocator,
{
    #[inline]
    fn from(value: rust_alloc::string::String) -> Self {
        Self {
            vec: Vec::from(value.into_bytes()),
        }
    }
}

impl<A> fmt::Display for FromUtf8Error<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl<A> Error for FromUtf8Error<A> where A: Allocator {}

impl<A, B> PartialEq<FromUtf8Error<B>> for FromUtf8Error<A>
where
    A: Allocator,
    B: Allocator,
{
    #[inline]
    fn eq(&self, other: &FromUtf8Error<B>) -> bool {
        self.bytes == other.bytes && self.error == other.error
    }
}

impl<A> fmt::Debug for FromUtf8Error<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromUtf8Error")
            .field("bytes", &self.bytes)
            .field("error", &self.error)
            .finish()
    }
}

impl<A> FromUtf8Error<A>
where
    A: Allocator,
{
    /// Returns a slice of [`u8`]s bytes that were attempted to convert to a
    /// `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut bytes = Vec::new_in(alloc);
    ///     // some invalid bytes, in a vector
    ///     bytes.extend_from_slice(&[0, 159])?;
    ///
    ///     let value = String::from_utf8(bytes);
    ///
    ///     assert_eq!(&[0, 159], value.unwrap_err().as_bytes());
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns the bytes that were attempted to convert to a `String`.
    ///
    /// This method is carefully constructed to avoid allocation. It will
    /// consume the error, moving out the bytes, so that a copy of the bytes
    /// does not need to be made.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut bytes = Vec::new_in(alloc);
    ///     // some invalid bytes, in a vector
    ///     bytes.extend_from_slice(&[0, 159])?;
    ///
    ///     let value = String::from_utf8(bytes);
    ///
    ///     assert_eq!(&[0, 159], value.unwrap_err().into_bytes());
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8, A> {
        self.bytes
    }

    /// Fetch a `Utf8Error` to get more details about the conversion failure.
    ///
    /// The [`Utf8Error`] type provided by [`std::str`] represents an error that
    /// may occur when converting a slice of [`u8`]s to a [`&str`]. In this
    /// sense, it's an analogue to `FromUtf8Error`. See its documentation for
    /// more details on using it.
    ///
    /// [`std::str`]: core::str "std::str"
    /// [`&str`]: prim@str "&str"
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Vec, String};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut bytes = Vec::new_in(alloc);
    ///     // some invalid bytes, in a vector
    ///     bytes.extend_from_slice(&[0, 159])?;
    ///
    ///     let value = String::from_utf8(bytes);
    ///
    ///     let error = value.unwrap_err().utf8_error();
    ///     // the first byte is invalid here
    ///     assert_eq!(1, error.valid_up_to());
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn utf8_error(&self) -> Utf8Error {
        self.error
    }
}

impl<M, A> Encode<M> for String<A>
where
    A: Allocator,
{
    type Encode = str;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.as_str().encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

/// Decode implementation for a Müsli-allocated [`String`].
///
/// # Examples
///
/// ```
/// use musli::alloc::String;
/// use musli::{Allocator, Decode};
///
/// #[derive(Decode)]
/// struct Struct<A> where A: Allocator {
///     field: String<A>
/// }
/// ```
impl<'de, M, A> Decode<'de, M, A> for String<A>
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        struct Visitor;

        #[crate::trait_defaults(crate)]
        impl<C> UnsizedVisitor<'_, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = String<Self::Allocator>;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string")
            }

            #[inline]
            fn visit_owned(
                self,
                _: C,
                value: String<Self::Allocator>,
            ) -> Result<Self::Ok, C::Error> {
                Ok(value)
            }

            #[inline]
            fn visit_ref(self, cx: C, string: &str) -> Result<Self::Ok, C::Error> {
                let mut s = String::new_in(cx.alloc());
                s.push_str(string).map_err(cx.map())?;
                Ok(s)
            }
        }

        decoder.decode_string(Visitor)
    }
}
