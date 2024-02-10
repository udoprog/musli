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
use musli::Context;

use crate::de::StorageDecoder;
use crate::en::StorageEncoder;
use crate::error::Error;
use crate::fixed_bytes::FixedBytes;
use crate::int::ByteOrder;
use crate::options::{self, Options};
use crate::reader::{Reader, SliceReader};
use crate::writer::Writer;

const DEFAULT_OPTIONS: Options = options::new().build();

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: Encoding<DefaultMode> = Encoding::new();

/// Encode the given value to the given [`Writer`] using the [DEFAULT]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Writer,
    Error: From<W::Error>,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [Write][io::Write] using the [DEFAULT]
/// configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: io::Write,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [`Vec`] using the [DEFAULT] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, Error>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Reader] using the [DEFAULT]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
where
    R: Reader<'de>,
    Error: From<R::Error>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [DEFAULT]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
pub struct Encoding<M = DefaultMode, const F: Options = DEFAULT_OPTIONS>
where
    M: Mode,
{
    _marker: marker::PhantomData<M>,
}

impl Encoding<DefaultMode, DEFAULT_OPTIONS> {
    /// Construct a new [Encoding] instance which uses [Variable] integer
    /// encoding.
    ///
    /// You can modify this behavior by using a custom [Options] instance:
    ///
    /// ```rust
    /// use musli_storage::Encoding;
    /// use musli_storage::options::{self, Options, Integer};
    /// use musli::mode::DefaultMode;
    /// use musli::{Encode, Decode};
    ///
    /// const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<DefaultMode, OPTIONS> = Encoding::new().with_options();
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

impl<M, const F: Options> Encoding<M, F>
where
    M: Mode,
{
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T, F>
    where
        T: Mode,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Modify the flavor of the current encoding into `U`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_storage::Encoding;
    /// use musli_storage::options::{self, Options, Integer};
    /// use musli::mode::DefaultMode;
    ///
    /// const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<DefaultMode, OPTIONS> = Encoding::new().with_options();
    /// ```
    pub const fn with_options<const U: Options>(self) -> Encoding<M, U> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable integer encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_variable_integers(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_integers(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer network-endian encoding
    /// (Default).
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_integers_ne(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer little-endian encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_integers_le(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer big-endian encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_integers_be(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer custom endian encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_integers_endian<E>(self) -> Encoding<M, F>
    where
        E: ByteOrder,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable length encoding.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_variable_lengths(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 32-bit encoding when encoding
    /// lengths.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_lengths(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 64-bit encoding when encoding
    /// lengths.
    #[deprecated = "This does nothing, use `with_options` instead"]
    pub const fn with_fixed_lengths64(self) -> Encoding<M, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    musli_common::encoding_impls!(
        StorageEncoder::<_, F, _>::new,
        StorageDecoder::<_, F, _>::new
    );
    musli_common::encoding_from_slice_impls!(StorageDecoder::<_, F, _>::new);
}

impl<M, const F: Options> Clone for Encoding<M, F>
where
    M: Mode,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, const F: Options> Copy for Encoding<M, F> where M: Mode {}
