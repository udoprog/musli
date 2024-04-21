//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [`DEFAULT`] encoding configuration.

use core::marker;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::io;

use musli_core::mode::Binary;
use musli_core::{Context, Decode, Encode};

use crate::options;
use crate::{FixedBytes, Options, Reader, Writer};

use super::de::WireDecoder;
use super::en::WireEncoder;
use super::error::Error;

/// The default flavor used by the [`DEFAULT`] configuration.
pub const OPTIONS: options::Options = options::new().build();

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [`zigzag`] with [`variable length`]
/// encoding for numbers.
///
/// [`zigzag`]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding
/// [`variable length`]: https://en.wikipedia.org/wiki/Variable-length_quantity
pub const DEFAULT: Encoding = Encoding::new();

/// Encode the given value to the given [`Writer`] using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Writer,
    T: ?Sized + Encode<Binary>,
{
    DEFAULT.encode(writer, value)
}

/// Encode the given value to the given [`Write`] using the [`DEFAULT`]
/// configuration.
///
/// [`Write`]: io::Write
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: io::Write,
    T: ?Sized + Encode<Binary>,
{
    DEFAULT.to_writer(writer, value)
}

/// Encode the given value to a [Vec] using the [`DEFAULT`] configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Encode<Binary>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, Error>
where
    T: ?Sized + Encode<Binary>,
{
    DEFAULT.to_fixed_bytes::<N, _>(value)
}

/// Decode the given type `T` from the given [`Reader`] using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
where
    R: Reader<'de>,
    T: Decode<'de, Binary>,
{
    DEFAULT.decode(reader)
}

/// Decode the given type `T` from the given slice using the [`DEFAULT`]
/// configuration.
#[inline]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Decode<'de, Binary>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
pub struct Encoding<const OPT: Options = OPTIONS, M = Binary> {
    _marker: marker::PhantomData<M>,
}

impl Encoding<OPTIONS, Binary> {
    /// Construct a new [`Encoding`] instance with the [`OPTIONS`]
    /// configuration.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::wire::Encoding;
    /// use musli::options::{self, Options, Integer};
    ///
    /// const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
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

impl<const OPT: Options, M> Encoding<OPT, M> {
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::wire::{OPTIONS, Encoding};
    ///
    /// enum Custom {}
    ///
    /// const CONFIG: Encoding<OPTIONS, Custom> = Encoding::new().with_mode();
    /// ```
    pub const fn with_mode<T>(self) -> Encoding<OPT, T> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Change the options of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options, Integer};
    /// use musli::wire::Encoding;
    ///
    /// const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
    /// ```
    pub const fn with_options<const U: Options>(self) -> Encoding<U, M> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    crate::encoding_impls!(
        M,
        WireEncoder::<_, OPT, _>::new,
        WireDecoder::<_, OPT, _>::new
    );
    crate::encoding_from_slice_impls!(M);
}

impl<const OPT: Options, M> Clone for Encoding<OPT, M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const OPT: Options, M> Copy for Encoding<OPT, M> {}
