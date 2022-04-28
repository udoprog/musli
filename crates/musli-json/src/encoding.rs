//! Module that defines [JsonEncoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

#[cfg(feature = "std")]
use std::io;

use crate::de::JsonDecoder;
use crate::en::JsonEncoder;
use musli::Decode;
use musli::Encode;
use musli_binary_common::fixed_bytes::{FixedBytes, FixedBytesWriterError};
use musli_binary_common::reader::{Reader, SliceReader, SliceReaderError};
#[cfg(feature = "std")]
use musli_binary_common::writer::VecWriterError;
use musli_binary_common::writer::Writer;

/// The default configuration.
pub const DEFAULT: JsonEncoding = JsonEncoding::new();

/// Encode the given value to the given [Writer] using the [DEFAULT]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), W::Error>
where
    W: Writer,
    T: ?Sized + Encode,
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
    T: ?Sized + Encode,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [Vec] using the [DEFAULT] configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, VecWriterError>
where
    T: ?Sized + Encode,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, FixedBytesWriterError>
where
    T: ?Sized + Encode,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Reader] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Decode<'de>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, SliceReaderError>
where
    T: Decode<'de>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
#[derive(Clone, Copy)]
pub struct JsonEncoding {}

impl JsonEncoding {
    /// Construct a new [JsonEncoding].
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_json::JsonEncoding;
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: JsonEncoding = JsonEncoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(default_field_tag = "name")]
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
    /// // TODO: uncomment these.
    /// let out = CONFIG.to_vec(&expected)?;
    /// panic!("{}", String::from_utf8(out)?);
    /// // let actual = CONFIG.decode(&out[..])?;
    ///
    /// // assert_eq!(expected, actual);
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        JsonEncoding {}
    }
}

impl JsonEncoding {
    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode,
    {
        T::encode(value, JsonEncoder::new(&mut writer))
    }

    /// Encode the given value to the given [Write][io::Write] using the current
    /// configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode,
    {
        let mut writer = musli_binary_common::io::wrap(write);
        T::encode(value, JsonEncoder::new(&mut writer))
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, VecWriterError>
    where
        T: ?Sized + Encode,
    {
        let mut data = Vec::new();
        T::encode(value, JsonEncoder::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(
        self,
        value: &T,
    ) -> Result<FixedBytes<N>, FixedBytesWriterError>
    where
        T: ?Sized + Encode,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, JsonEncoder::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Reader] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Decode<'de>,
    {
        let mut reader = reader.with_position();
        T::decode(JsonDecoder::new(&mut reader))
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, SliceReaderError>
    where
        T: Decode<'de>,
    {
        let mut reader = SliceReader::new(bytes).with_position();
        T::decode(JsonDecoder::new(&mut reader))
    }
}
