//! Module that defines [JsonEncoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

use core::marker;
#[cfg(feature = "std")]
use std::io;

use crate::reader::{ParseError, Scratch};
use musli::{Decode, DefaultMode, Encode};
use musli_common::fixed_bytes::{FixedBytes, FixedBytesWriterError};
#[cfg(feature = "std")]
use musli_common::writer::VecWriterError;
use musli_common::writer::Writer;

use crate::de::JsonDecoder;
use crate::en::JsonEncoder;
use crate::reader::{Parser, SliceParser};

/// The default configuration.
pub const DEFAULT: JsonEncoding = JsonEncoding::new();

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

/// Encode the given value to a [Vec] using the [DEFAULT] configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, VecWriterError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, FixedBytesWriterError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Reader] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, ParseError>
where
    R: Parser<'de>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
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
#[derive(Clone, Copy)]
pub struct JsonEncoding<Mode = DefaultMode> {
    _mode: marker::PhantomData<Mode>,
}

impl<Mode> JsonEncoding<Mode> {
    /// Construct a new [JsonEncoding].
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_json::JsonEncoding;
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: JsonEncoding<Json> = JsonEncoding::new();
    ///
    /// // Mode marker indicating that some attributes should only apply when we're decoding in a JSON mode.
    /// enum Json {}
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(mode = Json, default_field_tag = "name")]
    /// struct Struct<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let expected = Struct {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// let out = CONFIG.to_vec(&expected)?;
    /// println!("{}", String::from_utf8(out)?);
    ///
    /// let out = musli_json::to_vec(&expected)?;
    /// println!("{}", String::from_utf8(out)?);
    /// // let actual = CONFIG.decode(&out[..])?;
    ///
    /// // assert_eq!(expected, actual);
    /// # Ok(()) }
    /// ```
    #[inline]
    pub const fn new() -> Self {
        JsonEncoding {
            _mode: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<Mode>,
    {
        T::encode(value, JsonEncoder::<Mode, _>::new(&mut writer))
    }

    /// Encode the given value to the given [Write][io::Write] using the current
    /// configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode<Mode>,
    {
        let mut writer = musli_common::io::wrap(write);
        T::encode(value, JsonEncoder::<Mode, _>::new(&mut writer))
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, VecWriterError>
    where
        T: ?Sized + Encode<Mode>,
    {
        let mut data = Vec::new();
        T::encode(value, JsonEncoder::<Mode, _>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_string<T>(self, value: &T) -> Result<String, VecWriterError>
    where
        T: ?Sized + Encode<Mode>,
    {
        let mut data = Vec::new();
        T::encode(value, JsonEncoder::<Mode, _>::new(&mut data))?;
        // SAFETY: Encoder is guaranteed to produce valid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(data) })
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(
        self,
        value: &T,
    ) -> Result<FixedBytes<N>, FixedBytesWriterError>
    where
        T: ?Sized + Encode<Mode>,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, JsonEncoder::<Mode, _>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Reader] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, mut reader: R) -> Result<T, ParseError>
    where
        R: Parser<'de>,
        T: Decode<'de, Mode>,
    {
        let mut scratch = Scratch::new();
        T::decode(JsonDecoder::new(&mut scratch, &mut reader))
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, ParseError>
    where
        T: Decode<'de, Mode>,
    {
        let mut scratch = Scratch::new();
        let mut reader = SliceParser::new(bytes);
        T::decode(JsonDecoder::new(&mut scratch, &mut reader))
    }
}
