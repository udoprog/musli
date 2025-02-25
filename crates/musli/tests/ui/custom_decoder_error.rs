use musli::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct Container<'a> {
    #[musli(with = "bytes")]
    pub data: &'a [u8],
}

mod bytes {
    use musli::{Decoder, Encoder};

    pub(crate) fn encode<E>(_this: &[u8], __encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        todo!()
    }

    pub(crate) fn decode<'de, D>(__decoder: D) -> Result<Vec<u8>, D::Error>
    where
        D: Decoder<'de>,
    {
        todo!()
    }
}

fn main() {}
