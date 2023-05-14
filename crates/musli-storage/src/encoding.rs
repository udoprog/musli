//! Module that defines [Encoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::marker;
#[cfg(feature = "std")]
use std::io;

use musli::de::Decode;
use musli::en::Encode;
use musli::mode::{DefaultMode, Mode};

use crate::de::StorageDecoder;
use crate::en::StorageEncoder;
use crate::error::BufferError;
use crate::fixed_bytes::FixedBytes;
use crate::int::{
    BigEndian, Fixed, FixedUsize, IntegerEncoding, LittleEndian, NetworkEndian, UsizeEncoding,
    Variable,
};
use crate::reader::{Reader, SliceReader};
use crate::writer::{Buffer, Writer};

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

/// Encode the given value to the given [`Writer`] using the [DEFAULT]
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

/// Encode the given value to a [Buffer] using the [DEFAULT] configuration.
#[inline]
pub fn to_buffer<T>(value: &T) -> Result<Buffer, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_buffer(value)
}

/// Encode the given value to a [Vec] using the [DEFAULT] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, BufferError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
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

/// Decode the given type `T` from the given [Reader] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, BufferError>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
pub struct Encoding<M = DefaultMode, I = Variable, L = Variable>
where
    M: Mode,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    _marker: marker::PhantomData<(M, I, L)>,
}

impl Encoding<DefaultMode, Variable, Variable> {
    /// Construct a new [Encoding] instance which uses [Variable] integer
    /// encoding.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_storage::Encoding;
    /// use musli_storage::int::{Fixed, Variable};
    /// use musli::mode::DefaultMode;
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: Encoding<DefaultMode, Fixed, Variable> = Encoding::new()
    ///     .with_fixed_integers();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Struct<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut out = Vec::new();
    ///
    /// let expected = Struct {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// CONFIG.encode(&mut out, &expected)?;
    /// let actual = CONFIG.decode(&out[..])?;
    ///
    /// assert_eq!(expected, actual);
    /// # Ok(()) }
    /// ```
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<M, I, L> Encoding<M, I, L>
where
    M: Mode,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T, Variable, L>
    where
        T: Mode,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable integer encoding.
    pub const fn with_variable_integers(self) -> Encoding<M, Variable, L> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer encoding.
    pub const fn with_fixed_integers(self) -> Encoding<M, Fixed, L> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer little-endian encoding.
    pub const fn with_fixed_integers_le(self) -> Encoding<M, Fixed<LittleEndian>, L> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer big-endian encoding.
    pub const fn with_fixed_integers_be(self) -> Encoding<M, Fixed<BigEndian>, L> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer network-endian encoding
    /// (Default).
    pub const fn with_fixed_integers_ne(self) -> Encoding<M, Fixed<NetworkEndian>, L> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable length encoding.
    pub const fn with_variable_lengths(self) -> Encoding<M, I, Variable> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 32-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths(self) -> Encoding<M, I, FixedUsize<u32>> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 64-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths64(self) -> Encoding<M, I, FixedUsize<u64>> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [`Writer`] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        T::encode(value, StorageEncoder::<_, I, L>::new(writer))
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
        let writer = crate::wrap::wrap(write);
        T::encode(value, StorageEncoder::<_, I, L>::new(writer))
    }

    /// Encode the given value to a [Buffer] using the current configuration.
    #[inline]
    pub fn to_buffer<T>(self, value: &T) -> Result<Buffer, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut data = Buffer::new();
        T::encode(value, StorageEncoder::<_, I, L>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        Ok(self.to_buffer(value)?.into_vec())
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(self, value: &T) -> Result<FixedBytes<N>, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, StorageEncoder::<_, I, L>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Reader] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Decode<'de, M>,
    {
        let reader = reader.with_position();
        T::decode(StorageDecoder::<_, I, L>::new(reader))
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, BufferError>
    where
        T: Decode<'de, M>,
    {
        let reader = SliceReader::new(bytes).with_position();
        T::decode(StorageDecoder::<_, I, L>::new(reader))
    }
}

impl<M, I, L> Clone for Encoding<M, I, L>
where
    M: Mode,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<M, I, L> Copy for Encoding<M, I, L>
where
    M: Mode,
    I: IntegerEncoding,
    L: UsizeEncoding,
{
}
