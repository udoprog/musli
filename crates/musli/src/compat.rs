//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use core::fmt;
use std::collections::VecDeque;
use std::marker;

use crate::de::ReferenceVisitor;
use crate::en::SequenceEncoder;
use crate::error::Error;
use crate::{Decode, Decoder, Encode, Encoder};

/// Ensures that the given value `T` is encoded as a sequence.
///
/// In contrast to the typical values that are encoded as sequences such as
/// [Vec], this can take a sequence by reference.
///
/// We must use a wrapper like this, because we can't provide an implementation
/// for `&[T]` since it would conflict with `&[u8]` which is specialized to
/// encode and decode as a byte array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sequence<T>(pub T);

impl<T> Sequence<T> {
    /// Construct a new sequence wrapper.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Encode for Sequence<&'_ [T]>
where
    T: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut seq = encoder.encode_sequence(self.0.len())?;

        for value in self.0 {
            let encoder = seq.next()?;
            T::encode(value, encoder)?;
        }

        seq.end()
    }
}

impl Encode for Sequence<()> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_sequence(0)?.end()
    }
}

impl<'de> Decode<'de> for Sequence<()> {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Self(decoder.decode_unit()?))
    }
}

/// Ensures that the given value `T` is encoded as bytes.
///
/// This is useful for values which have a generic implementation to be encoded
/// as a sequence, such as [Vec] and [VecDeque][std::collections::VecDeque].
///
/// We must use a wrapper like this, because we can't provide an implementation
/// for `Vec<T>` since it would conflict with `Vec<u8>` which is generalized to
/// encode as a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Bytes<T>(pub T);

impl Encode for Bytes<Vec<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.0.as_slice())
    }
}

impl<'de> Decode<'de> for Bytes<Vec<u8>> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder
            .decode_bytes(Visitor(marker::PhantomData))
            .map(Bytes);

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ReferenceVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = [u8];
            type Ok = Vec<u8>;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes.to_vec())
            }
        }
    }
}

impl Encode for Bytes<VecDeque<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let (first, second) = self.0.as_slices();
        encoder.encode_bytes_vectored(&[first, second])
    }
}

impl<'de> Decode<'de> for Bytes<VecDeque<u8>> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Bytes::<Vec<u8>>::decode(decoder).map(|Bytes(bytes)| Bytes(VecDeque::from(bytes)))
    }
}

impl<const N: usize> Encode for Bytes<[u8; N]> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.0)
    }
}

impl<'de, const N: usize> Decode<'de> for Bytes<[u8; N]> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array().map(Self)
    }
}
