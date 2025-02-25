use core::marker;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::mode::Binary;
use crate::options;
use crate::{Context, Decode, Encode, IntoReader, IntoWriter, Options};

use super::de::StorageDecoder;
use super::en::StorageEncoder;
#[cfg(feature = "alloc")]
use super::error::Error;

/// The default options for the storage encoding.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [`zigzag`] with [`variable length`]
/// encoding for numbers.
///
/// [`zigzag`]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding
/// [`variable length`]: https://en.wikipedia.org/wiki/Variable-length_quantity
pub const OPTIONS: Options = options::new().build();

#[allow(unused)]
const DEFAULT: Encoding = Encoding::new();

crate::macros::bare_encoding!(Binary, DEFAULT, storage, IntoReader, IntoWriter);

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
    /// Construct a new [`Encoding`] instance which uses [`OPTIONS`].
    ///
    /// You can modify this behavior by using a custom [`Options`] instance:
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::options::{self, Options, Integer};
    /// use musli::storage::Encoding;
    /// # use musli::storage::Error;
    ///
    /// const OPTIONS: Options = options::new().integer(Integer::Fixed).build();
    /// const CONFIG: Encoding<OPTIONS> = Encoding::new().with_options();
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
    /// ```rust
    /// use musli::storage::{OPTIONS, Encoding};
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
    /// use musli::storage::Encoding;
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
        storage,
        StorageEncoder::<OPT, false, _, _, M>::new,
        StorageDecoder::<OPT, false, _, _, M>::new,
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
