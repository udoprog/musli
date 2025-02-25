use core::marker;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::mode::Binary;
use crate::options;
use crate::{Context, Decode, Encode, IntoReader, IntoWriter, Options};

use super::de::SelfDecoder;
use super::en::SelfEncoder;
#[cfg(feature = "alloc")]
use super::error::Error;

/// The default options for the descriptive encoding.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [`zigzag`] with [`variable length`]
/// encoding for numbers.
///
/// [`zigzag`]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding
/// [`variable length`]: https://en.wikipedia.org/wiki/Variable-length_quantity
pub const OPTIONS: options::Options = options::new().build();

#[allow(unused)]
const DEFAULT: Encoding = Encoding::new();

crate::macros::bare_encoding!(Binary, DEFAULT, descriptive, IntoReader, IntoWriter);

/// Setting up encoding with parameters.
pub struct Encoding<const OPT: Options = OPTIONS, M = Binary>
where
    M: 'static,
{
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<OPTIONS, Binary> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding<OPTIONS, Binary> {
    /// Construct a new [`Encoding`] instance.
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::descriptive::Encoding;
    /// # use musli::descriptive::Error;
    ///
    /// const CONFIG: Encoding = Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut out = Vec::new();
    ///
    /// let expected = Person {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// CONFIG.encode(&mut out, &expected)?;
    /// let actual = CONFIG.decode(&out[..])?;
    ///
    /// assert_eq!(expected, actual);
    /// # Ok::<_, Error>(())
    /// ```
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<const OPT: Options, M> Encoding<OPT, M>
where
    M: 'static,
{
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::descriptive::{OPTIONS, Encoding};
    ///
    /// enum Custom {}
    ///
    /// const CONFIG: Encoding<OPTIONS, Custom> = Encoding::new().with_mode();
    /// ```
    pub const fn with_mode<T>(self) -> Encoding<OPT, T>
    where
        T: 'static,
    {
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
    /// use musli::descriptive::Encoding;
    ///
    /// const OPTIONS: Options = options::new().integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
    /// ```
    pub const fn with_options<const U: Options>(self) -> Encoding<U, M> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    crate::macros::encoding_impls!(
        M,
        descriptive,
        SelfEncoder::<OPT, _, _, M>::new,
        SelfDecoder::<OPT, _, _, M>::new,
        IntoReader::into_reader,
        IntoWriter::into_writer,
    );
}

impl<const OPT: Options, M> Clone for Encoding<OPT, M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const OPT: Options, M> Copy for Encoding<OPT, M> {}
