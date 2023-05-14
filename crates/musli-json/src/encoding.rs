//! Module that defines [Encoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

use core::marker;

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::io;

use musli::de::Decode;
use musli::en::Encode;
use musli::mode::{DefaultMode, Mode};

use crate::de::JsonDecoder;
use crate::en::JsonEncoder;
use crate::error::BufferError;
use crate::fixed_bytes::FixedBytes;
use crate::reader::{ParseError, Scratch};
use crate::reader::{Parser, SliceParser};
use crate::writer::{Buffer, Writer};

/// The default configuration.
pub const DEFAULT: Encoding = Encoding::new();

/// Encode the given value to the given [Writer] using the [DEFAULT]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), W::Error>
where
    W: Writer,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [Write][io::Write] using the [DEFAULT]
/// configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), io::Error>
where
    W: io::Write,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [`Buffer`] using the [DEFAULT] configuration.
#[inline]
pub fn to_buffer<T>(value: &T) -> Result<Buffer, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_buffer(value)
}

/// Encode the given value to a [`Vec`] using the [DEFAULT] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a [`String`] using the [DEFAULT] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_string(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Parser] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, ParseError>
where
    R: Parser<'de>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given string using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_str<'de, T>(string: &'de str) -> Result<T, ParseError>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_str(string)
}

/// Decode the given type `T` from the given slice using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, ParseError>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
pub struct Encoding<M = DefaultMode> {
    _marker: marker::PhantomData<M>,
}

impl<M> Encoding<M>
where
    M: Mode,
{
    /// Construct a new [Encoding].
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_json::Encoding;
    /// use musli::{Encode, Decode, Mode};
    ///
    /// const CONFIG: Encoding<Json> = Encoding::new();
    ///
    /// // Mode marker indicating that some attributes should
    /// // only apply when we're decoding in a JSON mode.
    /// enum Json {}
    ///
    /// impl Mode for Json {
    /// }
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(mode = Json, default_field_name = "name")]
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
    /// let out = CONFIG.to_buffer(&expected).unwrap();
    /// println!("{}", core::str::from_utf8(out.as_slice()).unwrap());
    ///
    /// let out = musli_json::to_buffer(&expected).unwrap();
    /// println!("{}", core::str::from_utf8(out.as_slice()).unwrap());
    /// let actual = musli_json::from_slice(out.as_slice()).unwrap();
    /// assert_eq!(expected, actual);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T>
    where
        T: Mode,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        T::encode(value, JsonEncoder::<M, _>::new(&mut writer))
    }

    /// Encode the given value to the given [Write][io::Write] using the current
    /// configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode<M>,
    {
        let mut writer = crate::wrap::wrap(write);
        T::encode(value, JsonEncoder::<M, _>::new(&mut writer))
    }

    /// Encode the given value to a [`Buffer`] using the current configuration.
    #[inline]
    pub fn to_buffer<T>(self, value: &T) -> Result<Buffer, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut data = Buffer::new();
        T::encode(value, JsonEncoder::<M, _>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a [`Vec`] using the current configuration.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        Ok(self.to_buffer(value)?.into_vec())
    }

    /// Encode the given value to a [`String`] using the current configuration.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_string<T>(self, value: &T) -> Result<String, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut data = Buffer::with_capacity(128);
        T::encode(value, JsonEncoder::<M, _>::new(&mut data))?;
        // SAFETY: Encoder is guaranteed to produce valid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(data.into_vec()) })
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(self, value: &T) -> Result<FixedBytes<N>, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, JsonEncoder::<M, _>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Parser] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, mut reader: R) -> Result<T, ParseError>
    where
        R: Parser<'de>,
        T: Decode<'de, M>,
    {
        let mut scratch = Scratch::new();
        T::decode(JsonDecoder::new(&mut scratch, &mut reader))
    }

    /// Decode the given type `T` from the given string using the current
    /// configuration.
    #[inline]
    pub fn from_str<'de, T>(self, string: &'de str) -> Result<T, ParseError>
    where
        T: Decode<'de, M>,
    {
        self.from_slice(string.as_bytes())
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, ParseError>
    where
        T: Decode<'de, M>,
    {
        let mut scratch = Scratch::new();
        let mut reader = SliceParser::new(bytes);
        T::decode(JsonDecoder::new(&mut scratch, &mut reader))
    }
}

impl<M> Clone for Encoding<M> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<M> Copy for Encoding<M> {}
