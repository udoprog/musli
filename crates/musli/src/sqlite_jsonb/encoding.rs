use core::marker;

#[cfg(feature = "alloc")]
use crate::alloc::Global;
use crate::mode::Text;
use crate::{Context, Decode, Encode, IntoWriter};

use super::cursor::IntoCursor;
use super::de::JsonbDecoder;
use super::en::JsonbEncoder;
#[cfg(feature = "alloc")]
use super::error::Error;

#[allow(unused)]
const DEFAULT: Encoding = Encoding::new();

crate::macros::bare_encoding!(Text, DEFAULT, sqlite_jsonb, IntoCursor, IntoWriter);

/// Setting up encoding with parameters.
pub struct Encoding<M = Text>
where
    M: 'static,
{
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<Text> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding<Text> {
    /// Construct a new [`Encoding`].
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::sqlite_jsonb::Encoding;
    /// # use musli::sqlite_jsonb::Error;
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
    #[inline]
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<M> Encoding<M>
where
    M: 'static,
{
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::sqlite_jsonb::Encoding;
    ///
    /// enum Custom {}
    ///
    /// const CONFIG: Encoding<Custom> = Encoding::new().with_mode();
    /// ```
    pub const fn with_mode<T>(self) -> Encoding<T>
    where
        T: 'static,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    crate::macros::encoding_impls!(
        M,
        sqlite_jsonb,
        JsonbEncoder::<_, _, M>::new,
        JsonbDecoder::<_, _, M>::new,
        IntoCursor::into_cursor,
        crate::sqlite_jsonb::Cursor,
        IntoWriter::into_writer,
    );
}

impl<M> Clone for Encoding<M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M> Copy for Encoding<M> {}
