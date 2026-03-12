use core::error::Error;
use std::vec::Vec;

use musli::alloc::Global;
use musli::mode::Binary;
use musli::reader::IntoReader;
use musli::storage;
use musli::{Decode, Encode};

/// Trait governing the serialization to use for transport.
pub trait Framework
where
    Self: 'static,
{
    type Error: 'static + Error;

    fn to_writer<T>(writer: &mut Vec<u8>, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Encode<Binary>;

    fn decode<'de, R, T>(reader: R) -> Result<T, Self::Error>
    where
        R: IntoReader<'de>,
        T: Decode<'de, Binary, Global>;
}

#[non_exhaustive]
pub struct Storage;

impl Framework for Storage {
    type Error = storage::Error;

    #[inline]
    fn to_writer<T>(writer: &mut Vec<u8>, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Encode<Binary>,
    {
        storage::to_writer(writer, value)
    }

    #[inline]
    fn decode<'de, R, T>(reader: R) -> Result<T, Self::Error>
    where
        R: IntoReader<'de>,
        T: Decode<'de, Binary, Global>,
    {
        storage::decode(reader)
    }
}

impl Clone for Storage {
    #[inline]
    fn clone(&self) -> Self {
        Self
    }
}

impl Copy for Storage {}
