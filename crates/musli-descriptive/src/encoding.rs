//! Module that defines [`Encoding`] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::marker;
use musli_common::context::Capture;
#[cfg(feature = "std")]
use std::io;

use musli::de::Decode;
use musli::en::Encode;
use musli::mode::{DefaultMode, Mode};

use crate::de::SelfDecoder;
use crate::en::SelfEncoder;
use crate::error::BufferError;
use crate::fixed_bytes::FixedBytes;
use crate::reader::{Reader, SliceReader};
use crate::tag::MAX_INLINE_LEN;
use crate::writer::{Buffer, Writer};

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// The maximum pack length permitted equals to [MAX_INLINE_LEN], which is 62.
/// Trying to encode larger packs will result in a runtime error. This can be
/// modified with [Encoding::with_max_pack].
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: Encoding = Encoding::new();

/// Encode the given value to the given [Writer] using the [DEFAULT]
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
pub struct Encoding<M = DefaultMode, const P: usize = MAX_INLINE_LEN> {
    _marker: marker::PhantomData<M>,
}

impl Encoding<DefaultMode, MAX_INLINE_LEN> {
    /// Construct a new [`Encoding`] instance.
    ///
    /// ```rust
    /// use musli_descriptive::{Encoding};
    /// use musli::{Encode, Decode};
    /// use musli::mode::DefaultMode;
    ///
    /// const CONFIG: Encoding<DefaultMode> = Encoding::new();
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

impl<M, const P: usize> Encoding<M, P>
where
    M: Mode,
{
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> Encoding<T, P>
    where
        T: Mode,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Modify the maximum pack sized allowwed in the wire format. This defaults
    /// to [MAX_INLINE_LEN], a value that will fit unencoded in the
    /// [Tag][crate::tag::Tag] of the type.
    pub const fn with_max_pack<const N: usize>(self) -> Encoding<M, N> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        T::encode(value, SelfEncoder::<_, P>::new(&mut writer))
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
        let mut writer = musli_common::wrap::wrap(write);
        T::encode(value, SelfEncoder::<_, P>::new(&mut writer))
    }

    /// Encode the given value to a [Buffer] using the current configuration.
    #[inline]
    pub fn to_buffer<T>(self, value: &T) -> Result<Buffer, BufferError>
    where
        T: ?Sized + Encode<M>,
    {
        let mut data = Buffer::new();
        T::encode(value, SelfEncoder::<_, P>::new(&mut data))?;
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
        T::encode(value, SelfEncoder::<_, P>::new(&mut bytes))?;
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
        let mut cx = Capture::default();

        let Ok(value) = T::decode(&mut cx, SelfDecoder::<_>::new(reader.with_position())) else {
            return Err(cx.unwrap());
        };

        Ok(value)
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, BufferError>
    where
        T: Decode<'de, M>,
    {
        let mut cx = Capture::default();

        let Ok(value) = T::decode(&mut cx, SelfDecoder::<_>::new(SliceReader::new(bytes).with_position())) else {
            return Err(cx.unwrap());
        };

        Ok(value)
    }
}

impl<M, const P: usize> Clone for Encoding<M, P>
where
    M: Mode,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<M, const P: usize> Copy for Encoding<M, P> where M: Mode {}
