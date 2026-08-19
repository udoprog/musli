use core::marker;

#[cfg(feature = "alloc")]
use rust_alloc::string::String;
#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::alloc::Global;
use crate::mode::Text;
use crate::{Context, Decode, Encode, IntoWriter, Writer};

use super::de::JsonDecoder;
use super::en::JsonEncoder;
#[cfg(feature = "alloc")]
use super::error::Error;
use super::parser::IntoParser;
use super::pretty_writer::{Pretty, PrettyWriter};

#[allow(unused)]
const DEFAULT: Encoding = Encoding::new();

#[allow(unused)]
const PRETTY: Encoding = Encoding::new().with_pretty(Pretty::new());

crate::macros::bare_encoding!(Text, DEFAULT, json, IntoParser, IntoWriter);

/// Encode the given value to a [`String`] using the default [`Encoding`].
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
///     age: 61,
/// })?;
///
/// let person: Person = json::from_str(&data[..])?;
/// assert_eq!(person.name, "Aristotle");
/// assert_eq!(person.age, 61);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Encode<Text>,
{
    DEFAULT.to_string(value)
}

/// Encode the given value to a pretty printed [`String`] using the default
/// [`Encoding`].
///
/// This is the same as [`to_string`], except that the output is indented using
/// two spaces per level of nesting.
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
/// let data = json::to_string_pretty(&Person {
///     name: "Aristotle".to_string(),
///     age: 61,
/// })?;
///
/// assert_eq!(data, r#"{
///   "name": "Aristotle",
///   "age": 61
/// }"#);
///
/// let person: Person = json::from_str(&data[..])?;
/// assert_eq!(person.name, "Aristotle");
/// assert_eq!(person.age, 61);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn to_string_pretty<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Encode<Text>,
{
    PRETTY.to_string(value)
}

/// Encode the given value to a pretty printed [`Vec`] using the default
/// [`Encoding`].
///
/// This is the same as [`to_vec`], except that the output is indented using two
/// spaces per level of nesting.
///
/// [`Vec`]: rust_alloc::vec::Vec
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
/// let data = json::to_vec_pretty(&Person {
///     name: "Aristotle".to_string(),
///     age: 61,
/// })?;
///
/// assert_eq!(data, br#"{
///   "name": "Aristotle",
///   "age": 61
/// }"#);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Encode<Text>,
{
    PRETTY.to_vec(value)
}

/// Encode the given value to the given slice as pretty printed JSON using the
/// default [`Encoding`] and return the number of bytes encoded.
///
/// This is the same as [`to_slice`], except that the output is indented using
/// two spaces per level of nesting.
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
/// let mut buf = [0u8; 128];
///
/// let w = json::to_slice_pretty(&mut buf[..], &Person {
///     name: "Aristotle".to_string(),
///     age: 61,
/// })?;
///
/// assert_eq!(&buf[..w], br#"{
///   "name": "Aristotle",
///   "age": 61
/// }"#);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn to_slice_pretty<T>(out: &mut [u8], value: &T) -> Result<usize, Error>
where
    T: ?Sized + Encode<Text>,
{
    PRETTY.to_slice(out, value)
}

/// Encode the given value as pretty printed JSON to the given [`std::io::Write`]
/// using the default [`Encoding`].
///
/// This is the same as [`to_writer`], except that the output is indented using
/// two spaces per level of nesting.
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
/// let mut data = Vec::new();
///
/// json::to_writer_pretty(&mut data, &Person {
///     name: "Aristotle".to_string(),
///     age: 61,
/// })?;
///
/// assert_eq!(data, br#"{
///   "name": "Aristotle",
///   "age": 61
/// }"#);
/// # Ok::<_, Error>(())
/// ```
#[cfg(all(feature = "std", feature = "alloc"))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
#[inline]
pub fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: std::io::Write,
    T: ?Sized + Encode<Text>,
{
    PRETTY.to_writer(writer, value)
}

/// Decode the given type `T` from the given string using the default
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
///     age: 61,
/// })?;
///
/// let person: Person = json::from_str(&data[..])?;
/// assert_eq!(person.name, "Aristotle");
/// assert_eq!(person.age, 61);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn from_str<'de, T>(string: &'de str) -> Result<T, Error>
where
    T: Decode<'de, Text, Global>,
{
    DEFAULT.from_str(string)
}

/// Setting up encoding with parameters.
///
/// Pretty printing is off by default and is configured with
/// [`Encoding::with_pretty`].
pub struct Encoding<M = Text>
where
    M: 'static,
{
    pretty: Option<Pretty>,
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
    /// use musli::json::{self, Encoding};
    /// # use musli::json::Error;
    ///
    /// const CONFIG: Encoding<Json> = Encoding::new().with_mode();
    ///
    /// // Mode marker indicating that some attributes should
    /// // only apply when we're decoding in a JSON mode.
    /// enum Json {}
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(mode = Json, name_all = "name")]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let expected = Person {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// let out = CONFIG.to_string(&expected)?;
    /// let out = json::to_string(&expected)?;
    /// let actual = json::from_str(&out)?;
    /// assert_eq!(expected, actual);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Encoding {
            pretty: None,
            _marker: marker::PhantomData,
        }
    }
}

impl<M> Encoding<M>
where
    M: 'static,
{
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
    pub const fn with_mode<T>(self) -> Encoding<T>
    where
        T: 'static,
    {
        Encoding {
            pretty: self.pretty,
            _marker: marker::PhantomData,
        }
    }

    /// Pretty print the output of this encoding using the default indentation
    /// of two spaces.
    ///
    /// The indentation and any future formatting knobs are carried by
    /// [`Pretty`], which is built in a constant context.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::json::{Encoding, Pretty};
    /// # use musli::json::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    /// const PRETTY: Encoding = Encoding::new().with_pretty(Pretty::new());
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let person = Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// };
    ///
    /// assert_eq!(ENCODING.to_string(&person)?, r#"{"name":"Aristotle","age":61}"#);
    ///
    /// assert_eq!(PRETTY.to_string(&person)?, r#"{
    ///   "name": "Aristotle",
    ///   "age": 61
    /// }"#);
    /// # Ok::<_, Error>(())
    /// ```
    pub const fn with_pretty(self, pretty: Pretty) -> Encoding<M> {
        Encoding {
            pretty: Some(pretty),
            ..self
        }
    }

    /// Write compact output, which is the default.
    ///
    /// This undoes a previous [`Encoding::with_pretty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::json::{Encoding, Pretty};
    /// # use musli::json::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new()
    ///     .with_pretty(Pretty::new())
    ///     .with_compact();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let data = ENCODING.to_string(&Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// assert_eq!(data, r#"{"name":"Aristotle","age":61}"#);
    /// # Ok::<_, Error>(())
    /// ```
    pub const fn with_compact(self) -> Encoding<M> {
        Encoding {
            pretty: None,
            ..self
        }
    }

    /// Encode `value` into `writer`, picking the writer implementation which
    /// matches how this encoding is configured.
    #[inline]
    fn encode_into<C, W, T>(self, cx: C, writer: W, value: &T) -> Result<(), C::Error>
    where
        C: Context,
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        match self.pretty {
            Some(pretty) => T::encode(
                value,
                JsonEncoder::new(cx, PrettyWriter::new(writer, pretty)),
            ),
            None => T::encode(value, JsonEncoder::new(cx, writer)),
        }
    }

    crate::macros::encoding_impls!(
        custom,
        M,
        json,
        JsonDecoder::<_, _, M>::new,
        IntoParser::into_parser,
        crate::json::parser::Parser,
        IntoWriter::into_writer,
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
    ///     age: 61,
    /// })?;
    ///
    /// let person: Person = ENCODING.from_str(&data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 61);
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn to_string<T>(self, value: &T) -> Result<String, Error>
    where
        T: ?Sized + Encode<M>,
    {
        let cx = crate::context::new().with_error();
        self.to_string_with(&cx, value)
    }

    /// Encode the given value to the given value to a [`String`] using the
    /// current [`Encoding`].
    ///
    /// This is the same as [`Encoding::to_string`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::json;
    /// use musli::alloc::Global;
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
    /// let cx = musli::context::new().with_error();
    ///
    /// let mut data = ENCODING.to_string_with(&cx, &Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// let person: Person = ENCODING.from_str_with(&cx, &data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 61);
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn to_string_with<T, C>(self, cx: C, value: &T) -> Result<String, C::Error>
    where
        C: Context,
        T: ?Sized + Encode<M>,
    {
        cx.clear();
        let mut data = Vec::with_capacity(128);
        self.encode_into(cx, &mut data, value)?;
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
