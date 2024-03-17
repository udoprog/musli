use musli::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct Container<'a> {
    #[musli(with = "bytes")]
    pub data: &'a [u8],
}

mod bytes {
    use musli::{Decoder, Encoder};
    use musli::Context;

    pub(crate) fn encode<C, E>(this: &[u8], cx: &C, mut encoder: E) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
        E: Encoder<C>,
    {
        todo!()
    }

    pub(crate) fn decode<'de, C, D>(cx: &C, mut decoder: D) -> Result<Vec<u8>, C::Error>
    where
        C: ?Sized + Context,
        D: Decoder<'de, C>,
    {
        todo!()
    }
}

fn main() {
}
