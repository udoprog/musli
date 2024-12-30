use musli::{Encode, Decode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(with = self::array)]
    field: [u32; 4],
}

mod array {
    use musli::{Encoder, Decoder};

    #[inline]
    fn encode<E, T, const N: usize>(_this: &[T; N], __encoder: E) -> Result<E::Ok, E::Error>
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

fn main() {
}
