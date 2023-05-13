use musli::{Encode, Decode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(with = self::array::<_, 4>)]
    field: [u32; 4],
}

mod array {
    use musli::{Mode, Encoder, Decoder};

    #[inline]
    fn encode<M, E, T, const N: usize>(this: &[T; N], encoder: E) -> Result<E::Ok, E::Error>
    where
        M: Mode,
        E: Encoder,
    {
        todo!()
    }

    #[inline]
    fn decode<'de, M, D, T, const N: usize>(decoder: D) -> Result<[T; N], D::Error>
    where
        M: Mode,
        D: Decoder<'de>,
    {
        todo!()
    }
}

fn main() {
}
