use musli::{Encode, Decode};

#[derive(Decode, Encode)]
struct Struct {
    #[musli(with = self::array)]
    field: [u32; 4],
}

mod array {
    use musli::{Context, Encoder, Decoder};

    #[inline]
    fn encode<C, E, T, const N: usize>(this: &[T; N], cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context,
        E: Encoder<C>,
    {
        todo!()
    }

    #[inline]
    fn decode<'de, C, D, T, const N: usize>(cx: &C, decoder: D) -> Result<[T; N], C::Error>
    where
        C: ?Sized + Context,
        D: Decoder<'de, C>,
    {
        todo!()
    }
}

fn main() {
}
