use crate::Context;

use super::{Decode, DecodeSliceBuilder, Decoder, SequenceDecoder};

/// Default implementation to decode a slice.
#[inline]
pub fn default_decode_slice<'de, D, V, T>(decoder: D) -> Result<V, D::Error>
where
    D: Decoder<'de>,
    V: DecodeSliceBuilder<T>,
    T: Decode<'de, D::Mode, <D::Cx as Context>::Allocator>,
{
    use crate::Context;

    let cx = decoder.cx();

    decoder.decode_sequence(move |seq| {
        let mut out = V::with_capacity(cx, crate::internal::size_hint::cautious(seq.size_hint()))?;
        let mut index = 0usize;

        while let Some(value) = seq.try_decode_next()? {
            cx.enter_sequence_index(index);
            let value = T::decode(value)?;
            out.push(cx, value)?;
            cx.leave_sequence_index();
            index = index.wrapping_add(1);
        }

        Ok(out)
    })
}
