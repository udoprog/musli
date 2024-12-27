use core::fmt;

use crate::mode::Binary;
use crate::{Decode, Encode, IntoReader, Writer};

#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

/// Errors raised during api serialization.
pub struct Error(crate::storage::Error);

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for Error {}

/// Encode an API frame.
pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
where
    W: Writer,
    T: ?Sized + Encode<Binary>,
{
    crate::storage::encode(writer, value).map_err(Error)
}

/// Encode an API frame into an allocated vector.
#[cfg(feature = "alloc")]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Encode<Binary>,
{
    crate::storage::to_vec(value).map_err(Error)
}

/// Decode an API frame.
#[inline]
pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
where
    R: IntoReader<'de>,
    T: Decode<'de, Binary>,
{
    crate::storage::decode(reader).map_err(Error)
}

/// Decode an API frame from a slice.
#[cfg(feature = "alloc")]
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Decode<'de, Binary>,
{
    crate::storage::from_slice(bytes).map_err(Error)
}
