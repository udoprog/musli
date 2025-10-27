
#![cfg(feature = "garde022")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "garde022")))]

use std::ops::Deref;
use garde022::{Unvalidated, Valid, Validate};
use crate::{Decode, Decoder, Encode, Encoder, Allocator, Context};

impl<TValid, M> Encode<M> for Valid<TValid>
where
    TValid: Validate + Encode<M>
{
    const IS_BITWISE_ENCODE: bool = TValid::IS_BITWISE_ENCODE;
    type Encode = TValid::Encode;

    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode=M>
    {
        TValid::encode(self.deref(), encoder)
    }

    fn size_hint(&self) -> Option<usize> {
        TValid::size_hint(self.deref())
    }

    fn as_encode(&self) -> &Self::Encode {
        TValid::as_encode(self.deref())
    }
}

impl<'de, TValid, M, A> Decode<'de, M, A> for Unvalidated<TValid>
where
    TValid: Validate + Decode<'de, M, A>,
    A: Allocator
{
    const IS_BITWISE_DECODE: bool = TValid::IS_BITWISE_DECODE;

    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode=M, Allocator=A>
    {
        TValid::decode(decoder).map(Unvalidated::new)
    }
}


impl<'de, TValid, M, A> Decode<'de, M, A> for Valid<TValid>
where
    TValid: Validate<Context: Default> + Decode<'de, M, A>,
    A: Allocator
{
    const IS_BITWISE_DECODE: bool = TValid::IS_BITWISE_DECODE;

    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode=M, Allocator=A>
    {
        let cx = decoder.cx().clone();
        match Unvalidated::<TValid>::decode(decoder) {
            Ok(unvalidated) => match unvalidated.validate() {
                Ok(valid) => Ok(valid),
                Err(report) => Err(cx.custom(report)),
            }
            Err(e) => Err(e),
        }
    }
}

