use crate::hint::SequenceHint;
use crate::Context;

use super::{Encode, Encoder, SequenceEncoder};

/// Default implementation to encode a slice.
#[inline]
pub fn default_encode_slice<E, T>(encoder: E, slice: &[T]) -> Result<E::Ok, E::Error>
where
    E: Encoder,
    T: Encode<E::Mode>,
{
    encoder.cx(|cx, encoder| {
        let hint = SequenceHint::with_size(slice.len());
        let mut seq = encoder.encode_sequence(&hint)?;

        for (index, item) in slice.iter().enumerate() {
            cx.enter_sequence_index(index);
            seq.push(item)?;
            cx.leave_sequence_index();
        }

        seq.finish_sequence()
    })
}