//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [`DEFAULT`] encoding configuration.

use core::marker;

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::allocator;
use crate::de::{Decode, Decoder};
use crate::en::{Encode, Encoder};
use crate::mode::Text;
use crate::{Context, Writer};

use super::de::JsonDecoder;
use super::en::JsonEncoder;
use super::error::Error;
use super::parser::IntoParser;

/// The default configuration.
pub const DEFAULT: Encoding = Encoding::new();

crate::macros::bare_encoding!(Text, DEFAULT, json, IntoParser);

/// Encode the given value to a [`String`] using the [`DEFAULT`] [`Encoding`].
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::json;
/// # use musli::json::Error;
///
/// #[derive(Decode, Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let data = json::to_string(&Person {
///     name: "Aristotle".to_string(),
///     age: 62,
/// })?;
///
/// let person: Person = json::from_str(&data[..])?;
/// assert_eq!(person.name, "Aristotle");
/// assert_eq!(person.age, 62);
/// # Ok::<(), Error>(())
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Encode<Text>,
{
    DEFAULT.to_string(value)
}

/// Decode the given type `T` from the given string using the [`DEFAULT`]
/// [`Encoding`].
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::json;
/// # use musli::json::Error;
///
/// #[derive(Decode, Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let mut data = json::to_string(&Person {
///     name: "Aristotle".to_string(),
///     age: 62,
/// })?;
///
/// let person: Person = json::from_str(&data[..])?;
/// assert_eq!(person.name, "Aristotle");
/// assert_eq!(person.age, 62);
/// # Ok::<(), Error>(())
/// ```
#[inline]
pub fn from_str<'de, T>(string: &'de str) -> Result<T, Error>
where
    T: Decode<'de, Text>,
{
    DEFAULT.from_str(string)
}

/// Setting up encoding with parameters.
pub struct Encoding<M = Text> {
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<Text> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding<Text> {
    /// Construct a new [`Encoding`].
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::json::Encoding;
    ///
    /// const CONFIG: Encoding<Json> = Encoding::new().with_mode();
    ///
    /// // Mode marker indicating that some attributes should
    /// // only apply when we're decoding in a JSON mode.
    /// enum Json {}
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(mode = Json, name_all = "name")]
    /// struct Struct<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let expected = Struct {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// let out = CONFIG.to_vec(&expected).unwrap();
    /// println!("{}", core::str::from_utf8(out.as_slice()).unwrap());
    ///
    /// let out = musli::json::to_vec(&expected).unwrap();
    /// println!("{}", core::str::from_utf8(out.as_slice()).unwrap());
    /// let actual = musli::json::from_slice(out.as_slice()).unwrap();
    /// assert_eq!(expected, actual);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<M> Encoding<M> {
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::json::Encoding;
    ///
    /// enum Custom {}
    ///
    /// const CONFIG: Encoding<Custom> = Encoding::new().with_mode();
    /// ```
    pub const fn with_mode<T>(self) -> Encoding<T> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [`Writer`] using the current
    /// configuration.
    ///
    /// This is the same as [`Encoding::encode`] but allows for using a
    /// configurable [`Context`].
    #[inline]
    pub fn encode_with<C, W, T>(self, cx: &C, writer: W, value: &T) -> Result<(), C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        cx.clear();
        JsonEncoder::new(cx, writer).encode(value)
    }

    /// Encode the given value to a [`String`] using the current configuration.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_string<T>(self, value: &T) -> Result<String, Error>
    where
        T: ?Sized + Encode<M>,
    {
        allocator::default!(|alloc| {
            let cx = crate::context::Same::new(alloc);
            self.to_string_with(&cx, value)
        })
    }

    /// Encode the given value to a [`String`] using the current configuration.
    ///
    /// This is the same as [`Encoding::to_string`] but allows for using a
    /// configurable [`Context`].
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_string_with<T, C>(self, cx: &C, value: &T) -> Result<String, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        T: ?Sized + Encode<M>,
    {
        cx.clear();
        let mut data = Vec::with_capacity(128);
        JsonEncoder::new(cx, &mut data).encode(value)?;
        // SAFETY: Encoder is guaranteed to produce valid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(data) })
    }

    /// Decode the given type `T` from the given [`Parser`] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, P, T>(self, parser: P) -> Result<T, Error>
    where
        P: IntoParser<'de>,
        T: Decode<'de, M>,
    {
        allocator::default!(|alloc| {
            let cx = crate::context::Same::new(alloc);
            self.decode_with(&cx, parser)
        })
    }

    /// Decode the given type `T` from the given [`Parser`] using the current
    /// configuration.
    ///
    /// This is the same as [`Encoding::decode`] but allows for using a
    /// configurable [`Context`].
    #[inline]
    pub fn decode_with<'de, C, P, T>(self, cx: &C, parser: P) -> Result<T, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        P: IntoParser<'de>,
        T: Decode<'de, M>,
    {
        cx.clear();
        let parser = parser.into_parser();
        JsonDecoder::new(cx, parser).decode()
    }

    /// Decode the given type `T` from the given string using the current
    /// configuration.
    #[inline]
    pub fn from_str<'de, T>(self, string: &'de str) -> Result<T, Error>
    where
        T: Decode<'de, M>,
    {
        self.from_slice(string.as_bytes())
    }

    /// Decode the given type `T` from the given string using the current
    /// configuration.
    ///
    /// This is the same as [`Encoding::from_str`] but allows for using a
    /// configurable [`Context`].
    #[inline]
    pub fn from_str_with<'de, C, T>(self, cx: &C, string: &'de str) -> Result<T, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        T: Decode<'de, M>,
    {
        self.from_slice_with(cx, string.as_bytes())
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de, M>,
    {
        allocator::default!(|alloc| {
            let cx = crate::context::Same::<_, M, _>::new(alloc);
            self.from_slice_with(&cx, bytes)
        })
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    ///
    /// This is the same as [`Encoding::from_slice`] but allows for using a
    /// configurable [`Context`].
    #[inline]
    pub fn from_slice_with<'de, C, T>(self, cx: &C, bytes: &'de [u8]) -> Result<T, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        T: Decode<'de, M>,
    {
        cx.clear();
        JsonDecoder::new(cx, bytes.into_parser()).decode()
    }

    crate::macros::encode_with_extensions!(M, json);
}

impl<M> Clone for Encoding<M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M> Copy for Encoding<M> {}
