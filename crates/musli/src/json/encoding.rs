//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [`DEFAULT`] encoding configuration.

use core::marker;

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::allocator;
use crate::mode::Text;
use crate::{Context, Decode, Encode};

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

    crate::macros::encoding_impls!(
        M,
        json,
        JsonEncoder::new,
        JsonDecoder::new,
        IntoParser::into_parser
    );

    /// Encode the given value to the given value to a [`String`] using the
    /// current [`Encoding`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::json;
    /// # use musli::json::Error;
    ///
    /// const ENCODING: json::Encoding = json::Encoding::new();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let mut data = ENCODING.to_string(&Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 62,
    /// })?;
    ///
    /// let person: Person = ENCODING.from_str(&data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 62);
    /// # Ok::<(), Error>(())
    /// ```
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
        T::encode(value, cx, JsonEncoder::new(cx, &mut data))?;
        // SAFETY: Encoder is guaranteed to produce valid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(data) })
    }
}

impl<M> Clone for Encoding<M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M> Copy for Encoding<M> {}
