//! Type that implements a very simple encoding which utilizes length-prefixed
//! records where appropriate.
//!
//! The simple encoding format uses the following principle:
//! * Anything with variable length is *length prefixed*. The length is encoded
//!   using a [variable-length encoder][crate::int::continuation].
//! * A struct / tuple / variant is length-prefixed with the number of elements
//!   it contains.
//! * Each tagged record is prefixed with the tag of the record.

use core::marker;
#[cfg(feature = "std")]
use std::io;

use crate::de::StorageDecoder;
use crate::en::StorageEncoder;
use crate::integer_encoding::{Fixed, FixedLength, IntegerEncoding, UsizeEncoding, Variable};
use musli::Decode;
use musli::Encode;
use musli_binary_common::fixed_bytes::{FixedBytes, FixedBytesWriterError};
use musli_binary_common::int::{BigEndian, LittleEndian, NetworkEndian};
use musli_binary_common::reader::Reader;
#[cfg(feature = "std")]
use musli_binary_common::writer::VecWriterError;
use musli_binary_common::writer::Writer;

/// Default encoding to use.
const DEFAULT: StorageEncoding<Variable, Variable> = StorageEncoding::new();

/// Encode the given value to the given [Writer] using [StorageEncoder] with default
/// settings as defined by [StorageEncoding::new].
///
/// The default configuration uses [Variable] integer encoding.
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), W::Error>
where
    W: Writer,
    T: ?Sized + Encode,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [Write][std::io::Write] using
/// [StorageEncoder] with default settings as defined by [StorageEncoding::new].
#[cfg(feature = "std")]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), io::Error>
where
    W: io::Write,
    T: ?Sized + Encode,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [Vec] using [StorageEncoder] with default
/// settings as defined by [StorageEncoding::new].
#[cfg(feature = "std")]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, VecWriterError>
where
    T: ?Sized + Encode,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size byte storage using [StorageEncoder]
/// with default settings as defined by [StorageEncoding::new].
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, FixedBytesWriterError>
where
    T: ?Sized + Encode,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type from the given `reader` using [StorageEncoder] with
/// default settings as defined by [StorageEncoding::new].
pub fn decode<'de, R, T>(reader: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: Decode<'de>,
{
    DEFAULT.decode(reader)
}

/// Setting up encoding with parameters.
#[derive(Clone, Copy)]
pub struct StorageEncoding<I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    _marker: marker::PhantomData<(I, L)>,
}

impl StorageEncoding<Variable, Variable> {
    /// Construct a new [StorageEncoding] instance which uses [Variable] integer
    /// encoding.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_storage::{StorageEncoding, Fixed, Variable};
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: StorageEncoding<Fixed, Variable> = StorageEncoding::new()
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
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<I, L> StorageEncoding<I, L>
where
    I: IntegerEncoding,
    L: UsizeEncoding,
{
    /// Configure the encoding to use variable integer encoding.
    pub const fn with_variable_integers(self) -> StorageEncoding<Variable, L> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer encoding.
    pub const fn with_fixed_integers(self) -> StorageEncoding<Fixed, L> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer little-endian encoding.
    pub const fn with_fixed_integers_le(self) -> StorageEncoding<Fixed<LittleEndian>, L> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer big-endian encoding.
    pub const fn with_fixed_integers_be(self) -> StorageEncoding<Fixed<BigEndian>, L> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer network-endian encoding
    /// (Default).
    pub const fn with_fixed_integers_ne(self) -> StorageEncoding<Fixed<NetworkEndian>, L> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable length encoding.
    pub const fn with_variable_lengths(self) -> StorageEncoding<I, Variable> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 32-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths(self) -> StorageEncoding<I, FixedLength<u32>> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 64-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths64(self) -> StorageEncoding<I, FixedLength<u64>> {
        StorageEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using [StorageEncoder] with
    /// the current settings.
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode,
    {
        T::encode(&value, StorageEncoder::<_, I, L>::new(&mut writer))
    }

    /// Encode the given value to the given [Write][io::Write] using
    /// [StorageEncoder] with the current settings.
    #[cfg(feature = "std")]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode,
    {
        let mut writer = musli_binary_common::io::wrap(write);
        T::encode(&value, StorageEncoder::<_, I, L>::new(&mut writer))
    }

    /// Encode the given value to a [Vec] using [StorageEncoder] with the current
    /// settings.
    #[cfg(feature = "std")]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, VecWriterError>
    where
        T: ?Sized + Encode,
    {
        let mut data = Vec::new();
        T::encode(&value, StorageEncoder::<_, I, L>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a fixed-size bytes storage.
    pub fn to_fixed_bytes<const N: usize, T>(
        self,
        value: &T,
    ) -> Result<FixedBytes<N>, FixedBytesWriterError>
    where
        T: ?Sized + Encode,
    {
        let mut bytes = FixedBytes::new();
        T::encode(&value, StorageEncoder::<_, I, L>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type from the given `reader` using [StorageEncoder] with the
    /// current settings.
    pub fn decode<'de, R, T>(self, mut reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Decode<'de>,
    {
        T::decode(StorageDecoder::<_, I, L>::new(&mut reader))
    }
}
