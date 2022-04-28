//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use core::fmt;
use std::collections::VecDeque;
use std::marker;

use crate::compat::Bytes;
use crate::de::ValueVisitor;
use crate::error::Error;
use crate::{Decode, Decoder, Encode, Encoder};

impl<Mode> Encode<Mode> for Bytes<Vec<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.0.as_slice())
    }
}

impl<'de, Mode> Decode<'de, Mode> for Bytes<Vec<u8>> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder
            .decode_bytes(Visitor(marker::PhantomData))
            .map(Bytes);

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
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
            fn visit_borrowed(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes.to_vec())
            }

            #[inline]
            fn visit_any(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes.to_vec())
            }
        }
    }
}

impl<Mode> Encode<Mode> for Bytes<VecDeque<u8>> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let (first, second) = self.0.as_slices();
        encoder.encode_bytes_vectored(&[first, second])
    }
}

impl<'de, Mode> Decode<'de, Mode> for Bytes<VecDeque<u8>> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        <Bytes<Vec<u8>> as Decode<Mode>>::decode(decoder)
            .map(|Bytes(bytes)| Bytes(VecDeque::from(bytes)))
    }
}
