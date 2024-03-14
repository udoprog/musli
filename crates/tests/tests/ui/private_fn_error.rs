use musli::{Encode, Decode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(with = self::array::<_, 4>)]
    field: [u32; 4],
}

mod array {
    use musli::{Context, Mode, Encoder, Decoder};

    #[inline]
    fn encode<M, E, C, T, const N: usize>(this: &[T; N], cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        todo!()
    }

    #[inline]
    fn decode<'de, M, C, D, T, const N: usize>(cx: &C, decoder: D) -> Result<[T; N], C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        todo!()
    }
}

fn main() {
}
