//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use core::fmt;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::compat::Bytes;
use crate::de::{Decode, Decoder, ValueVisitor};
use crate::en::{Encode, Encoder};
use crate::mode::Mode;
use crate::Context;

impl<M> Encode<M> for Bytes<Vec<u8>>
where
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bytes(cx, self.0.as_slice())
    }
}

impl<'de, M> Decode<'de, M> for Bytes<Vec<u8>>
where
    M: Mode,
{
    #[inline]
    fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, 'buf, C> ValueVisitor<'de, 'buf, C, [u8]> for Visitor
        where
            C: Context<'buf>,
        {
            type Ok = Vec<u8>;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, _: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }

            #[inline]
            fn visit_ref(self, _: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }
        }

        decoder.decode_bytes(cx, Visitor).map(Bytes)
    }
}

impl<M> Encode<M> for Bytes<VecDeque<u8>>
where
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        let (first, second) = self.0.as_slices();
        encoder.encode_bytes_vectored(cx, &[first, second])
    }
}

impl<'de, M> Decode<'de, M> for Bytes<VecDeque<u8>>
where
    M: Mode,
{
    #[inline]
    fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = D::Error>,
        D: Decoder<'de>,
    {
        <Bytes<Vec<u8>> as Decode<M>>::decode(cx, decoder)
            .map(|Bytes(bytes)| Bytes(VecDeque::from(bytes)))
    }
}
