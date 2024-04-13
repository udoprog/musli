//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::marker;
#[cfg(feature = "std")]
use std::io;

use musli::de::Decode;
use musli::en::Encode;
use musli::mode::DefaultMode;
use musli::Context;
use musli_utils::fixed::FixedBytes;
use musli_utils::options::{self, Options};
use musli_utils::{Reader, Writer};

use crate::de::StorageDecoder;
use crate::en::StorageEncoder;
use crate::error::Error;

/// Default options to use with [`Encoding`].
pub const DEFAULT_OPTIONS: Options = options::new().build();

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// [zigzag]: musli_utils::int::zigzag
/// [continuation]: musli_utils::int::continuation
pub const DEFAULT: Encoding<DefaultMode> = Encoding::new();

/// Encode the given value to the given [`Writer`] using the [DEFAULT]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
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
pub struct Encoding<M = DefaultMode, const OPT: Options = DEFAULT_OPTIONS> {
    _marker: marker::PhantomData<M>,
}

impl Encoding<DefaultMode, DEFAULT_OPTIONS> {
    /// Construct a new [`Encoding`] instance which uses [`DEFAULT_OPTIONS`].
    ///
    /// You can modify this behavior by using a custom [`Options`] instance:
    ///
    /// ```rust
    /// use musli::mode::DefaultMode;
    /// use musli::{Encode, Decode};
    /// use musli_utils::options::{self, Options, Integer};
    /// use musli_storage::Encoding;
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

impl<M, const OPT: Options> Encoding<M, OPT> {
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T, OPT> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Modify the flavor of the current encoding into `U`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli::mode::DefaultMode;
    /// use musli_utils::options::{self, Options, Integer};
    /// use musli_storage::Encoding;
    ///
    /// const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<DefaultMode, OPTIONS> = Encoding::new().with_options();
    /// ```
    pub const fn with_options<const U: Options>(self) -> Encoding<M, U> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    musli_utils::encoding_impls!(
        M,
        StorageEncoder::<_, OPT, _>::new,
        StorageDecoder::<_, OPT, _>::new
    );
    musli_utils::encoding_from_slice_impls!(M);
}

impl<M, const OPT: Options> Clone for Encoding<M, OPT> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, const OPT: Options> Copy for Encoding<M, OPT> {}
