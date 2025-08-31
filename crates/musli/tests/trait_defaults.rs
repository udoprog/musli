use std::fmt;
use std::marker::PhantomData;

use musli::de::Decoder;
use musli::Context;

pub struct MyDecoder<C, M> {
    cx: C,
    _marker: PhantomData<M>,
}

#[musli::trait_defaults]
impl<'de, C, M> Decoder<'de> for MyDecoder<C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "32-bit unsigned integers")
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        Ok(42)
    }
}
