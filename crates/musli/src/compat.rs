//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use std::collections::VecDeque;
use std::marker;

use crate::de::BytesVisitor;
use crate::en::SequenceEncoder;
use crate::error::Error;
use crate::{Decode, Encode, Encoder};

/// Ensures that the given value `T` is encoded as a sequence.
///
/// In contrast to the typical values that are encoded as sequences such as
/// [Vec], this can take a sequence by reference.
///
/// We must use a wrapper like this, because we can't provide an implementation
/// for `&[T]` since it would conflict with `&[u8]` which is specialized to
/// encode and decode as a byte array.
pub struct Sequence<T> {
    value: T,
}

impl<T> Sequence<T> {
    /// Construct a new sequence wrapper.
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> Encode for Sequence<&'_ [T]>
where
    T: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut seq = encoder.encode_sequence(self.value.len())?;

        for value in self.value {
            let encoder = seq.encode_next()?;
            T::encode(value, encoder)?;
        }

        seq.finish()
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
pub struct Bytes<T>(pub T);

impl Encode for Bytes<Vec<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
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
        D: crate::Decoder<'de>,
    {
        return decoder
            .decode_bytes(Visitor(marker::PhantomData))
            .map(Bytes);

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> BytesVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Ok = Vec<u8>;
            type Error = E;

            #[inline]
            fn visit(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes.to_vec())
            }
        }
    }
}

impl Encode for Bytes<VecDeque<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
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
        D: crate::Decoder<'de>,
    {
        Bytes::<Vec<u8>>::decode(decoder).map(|Bytes(bytes)| Bytes(VecDeque::from(bytes)))
    }
}
