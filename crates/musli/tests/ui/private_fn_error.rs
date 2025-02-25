use musli::{Decode, Encode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(with = self::array)]
    field: [u32; 4],
}

mod array {
    use musli::{Decoder, Encoder};

    #[inline]
    fn encode<E, T, const N: usize>(_this: &[T; N], __encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        todo!()
    }

    #[inline]
    fn decode<'de, D, T, const N: usize>(__decoder: D) -> Result<[T; N], D::Error>
    where
        D: Decoder<'de>,
    {
        todo!()
    }
}

fn main() {}
