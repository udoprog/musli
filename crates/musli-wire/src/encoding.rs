//! Module that defines [WireEncoding] whith allows for customization of the
//! encoding format, and the [DEFAULT] encoding configuration.

use core::marker;
#[cfg(feature = "std")]
use std::io;

use crate::de::WireDecoder;
use crate::en::WireEncoder;
use crate::integer_encoding::{TypedIntegerEncoding, TypedUsizeEncoding};
use crate::tag::MAX_INLINE_LEN;
use musli::mode::DefaultMode;
use musli::{Decode, Encode, Mode};
use musli_common::encoding::{Fixed, FixedLength, Variable};
use musli_common::fixed_bytes::{FixedBytes, FixedBytesWriterError};
use musli_common::int::{BigEndian, LittleEndian, NetworkEndian};
use musli_common::reader::{Reader, SliceReader, SliceReaderError};
#[cfg(feature = "std")]
use musli_common::writer::VecWriterError;
use musli_common::writer::Writer;

/// The default configuration.
///
/// Uses variable-encoded numerical fields and variable-encoded prefix lengths.
///
/// The variable length encoding uses [zigzag] with [continuation] encoding for
/// numbers.
///
/// The maximum pack length permitted equals to [MAX_INLINE_LEN], which is 62.
/// Trying to encode larger packs will result in a runtime error. This can be
/// modified with [WireEncoding::with_max_pack].
///
/// [zigzag]: musli_common::int::zigzag
/// [continuation]: musli_common::int::continuation
pub const DEFAULT: WireEncoding = WireEncoding::new();

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

/// Encode the given value to a [Vec] using the [DEFAULT] configuration.
#[cfg(feature = "std")]
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, VecWriterError>
where
    T: ?Sized + Encode<DefaultMode>,
{
    DEFAULT.to_vec(value)
}

/// Encode the given value to a fixed-size bytes using the [DEFAULT]
/// configuration.
#[inline]
pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<FixedBytes<N>, FixedBytesWriterError>
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
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, SliceReaderError>
where
    T: Decode<'de, DefaultMode>,
{
    DEFAULT.from_slice(bytes)
}

/// Setting up encoding with parameters.
#[derive(Clone, Copy)]
pub struct WireEncoding<
    M = DefaultMode,
    I = Variable,
    L = Variable,
    const P: usize = MAX_INLINE_LEN,
> where
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    _marker: marker::PhantomData<(M, I, L)>,
}

impl WireEncoding<DefaultMode, Variable, Variable, MAX_INLINE_LEN> {
    /// Construct a new [WireEncoding] instance which uses [Variable] integer
    /// encoding.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```rust
    /// use musli_wire::{WireEncoding, Fixed, Variable};
    /// use musli::{Encode, Decode};
    ///
    /// const CONFIG: WireEncoding<Fixed, Variable> = WireEncoding::new()
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
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<M, const P: usize, I, L> WireEncoding<M, I, L, P>
where
    M: Mode,
    I: TypedIntegerEncoding,
    L: TypedUsizeEncoding,
{
    /// Change the mode of the encoding.
    pub const fn with_mode<T>(self) -> WireEncoding<T, I, L, P>
    where
        T: Mode,
    {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable integer encoding.
    pub const fn with_variable_integers(self) -> WireEncoding<M, Variable, L, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer encoding.
    pub const fn with_fixed_integers(self) -> WireEncoding<M, Fixed, L, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer little-endian encoding.
    pub const fn with_fixed_integers_le(self) -> WireEncoding<M, Fixed<LittleEndian>, L, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer big-endian encoding.
    pub const fn with_fixed_integers_be(self) -> WireEncoding<M, Fixed<BigEndian>, L, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed integer network-endian encoding
    /// (Default).
    pub const fn with_fixed_integers_ne(self) -> WireEncoding<M, Fixed<NetworkEndian>, L, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use variable length encoding.
    pub const fn with_variable_lengths(self) -> WireEncoding<M, I, Variable, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 32-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths(self) -> WireEncoding<M, I, FixedLength<u32>, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Configure the encoding to use fixed length 64-bit encoding when encoding
    /// lengths.
    pub const fn with_fixed_lengths64(self) -> WireEncoding<M, I, FixedLength<u64>, P> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Modify the maximum pack sized allowwed in the wire format. This defaults
    /// to [MAX_INLINE_LEN], a value that will fit unencoded in the
    /// [Tag][crate::tag::Tag] of the type.
    pub const fn with_max_pack<const N: usize>(self) -> WireEncoding<M, I, L, N> {
        WireEncoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode the given value to the given [Writer] using the current
    /// configuration.
    #[inline]
    pub fn encode<W, T>(self, mut writer: W, value: &T) -> Result<(), W::Error>
    where
        W: Writer,
        T: ?Sized + Encode<DefaultMode>,
    {
        T::encode(value, WireEncoder::<_, I, L, P>::new(&mut writer))
    }

    /// Encode the given value to the given [Write][io::Write] using the current
    /// configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
    where
        W: io::Write,
        T: ?Sized + Encode<DefaultMode>,
    {
        let mut writer = musli_common::io::wrap(write);
        T::encode(value, WireEncoder::<_, I, L, P>::new(&mut writer))
    }

    /// Encode the given value to a [Vec] using the current configuration.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, VecWriterError>
    where
        T: ?Sized + Encode<DefaultMode>,
    {
        let mut data = Vec::new();
        T::encode(value, WireEncoder::<_, I, L, P>::new(&mut data))?;
        Ok(data)
    }

    /// Encode the given value to a fixed-size bytes using the current
    /// configuration.
    #[inline]
    pub fn to_fixed_bytes<const N: usize, T>(
        self,
        value: &T,
    ) -> Result<FixedBytes<N>, FixedBytesWriterError>
    where
        T: ?Sized + Encode<DefaultMode>,
    {
        let mut bytes = FixedBytes::new();
        T::encode(value, WireEncoder::<_, I, L, P>::new(&mut bytes))?;
        Ok(bytes)
    }

    /// Decode the given type `T` from the given [Reader] using the current
    /// configuration.
    #[inline]
    pub fn decode<'de, R, T>(self, reader: R) -> Result<T, R::Error>
    where
        R: Reader<'de>,
        T: Decode<'de, DefaultMode>,
    {
        T::decode(WireDecoder::<_, I, L>::new(reader.with_position()))
    }

    /// Decode the given type `T` from the given slice using the current
    /// configuration.
    #[inline]
    pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, SliceReaderError>
    where
        T: Decode<'de, DefaultMode>,
    {
        T::decode(WireDecoder::<_, I, L>::new(
            SliceReader::new(bytes).with_position(),
        ))
    }
}
