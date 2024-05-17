//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [`DEFAULT`] encoding configuration.

use core::marker;

use crate::mode::Binary;
use crate::options;
use crate::{IntoReader, Options};

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

crate::macros::bare_encoding!(Binary, DEFAULT, wire, IntoReader);

/// Setting up encoding with parameters.
pub struct Encoding<const OPT: Options = OPTIONS, M = Binary> {
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<OPTIONS, Binary> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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

    crate::macros::encoding_impls!(
        M,
        wire,
        WireEncoder::<_, OPT, _>::new,
        WireDecoder::<_, OPT, _>::new,
        IntoReader::into_reader,
    );
}

impl<const OPT: Options, M> Clone for Encoding<OPT, M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const OPT: Options, M> Copy for Encoding<OPT, M> {}
