//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use core::fmt;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::compat::Bytes;
use crate::de::{Decode, Decoder, ValueVisitor};
use crate::en::{Encode, Encoder};
use crate::Context;

impl<M> Encode<M> for Bytes<Vec<u8>> {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bytes(cx, self.0.as_slice())
    }
}

impl<'de, M> Decode<'de, M> for Bytes<Vec<u8>> {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = Vec<u8>;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, _: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }

            #[inline]
            fn visit_ref(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }
        }

        decoder.decode_bytes(cx, Visitor).map(Bytes)
    }
}

impl<M> Encode<M> for Bytes<VecDeque<u8>> {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let (first, second) = self.0.as_slices();
        encoder.encode_bytes_vectored(cx, &[first, second])
    }
}

impl<'de, M> Decode<'de, M> for Bytes<VecDeque<u8>> {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        cx.decode(decoder)
            .map(|Bytes(bytes): Bytes<Vec<u8>>| Bytes(VecDeque::from(bytes)))
    }
}
