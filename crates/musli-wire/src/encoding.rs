//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [`DEFAULT`] encoding configuration.

use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::io;

use musli::de::Decode;
use musli::en::Encode;
use musli::mode::DefaultMode;
use musli::Context;

use crate::de::WireDecoder;
use crate::en::WireEncoder;
use crate::error::Error;
use crate::fixed_bytes::FixedBytes;
use crate::options::{self, Options};
use crate::reader::{Reader, SliceReader};
use crate::writer::Writer;

/// The default flavor used by the [`DEFAULT`] configuration.
pub const DEFAULT_OPTIONS: options::Options = options::new().build();

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: Encoding = Encoding::new();

/// Encode the given value to the given [Writer] using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Writer,
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [Write][io::Write] using the [`DEFAULT`]
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

/// Encode the given value to a [Vec] using the [`DEFAULT`] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, Error>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [Reader] using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
where
    R: Reader<'de>,
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
pub struct Encoding<M = DefaultMode, const F: Options = DEFAULT_OPTIONS> {
    _marker: marker::PhantomData<M>,
}

impl Encoding<DefaultMode, DEFAULT_OPTIONS> {
    /// Construct a new [`Encoding`] instance with the [`DEFAULT_OPTIONS`]
    /// configuration.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_wire::Encoding;
    /// use musli_wire::options::{self, Options, Integer};
    /// use musli::{Encode, Decode};
    /// use musli::mode::DefaultMode;
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

impl<M, const F: Options> Encoding<M, F> {
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T, F> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Modify the flavor of the current encoding into `U`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_wire::Encoding;
    /// use musli_wire::options::{self, Options, Integer};
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

    musli_common::encoding_impls!(M, WireEncoder::<_, F>::new, WireDecoder::<_, F>::new);
    musli_common::encoding_from_slice_impls!(M, WireDecoder::<_, F>::new);
}

impl<M, const F: Options> Clone for Encoding<M, F> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, const F: Options> Copy for Encoding<M, F> {}
