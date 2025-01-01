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

#[inline]
pub fn default_sequence_encode_slice<E, T>(
    seq: &mut E,
    slice: &[T],
) -> Result<(), <E::Cx as Context>::Error>
where
    E: ?Sized + SequenceEncoder,
    T: Encode<<E::Cx as Context>::Mode>,
{
    seq.cx_mut(|cx, this| {
        let mut index = 0usize;

        for value in slice {
            cx.enter_sequence_index(index);
            this.push(value)?;
            cx.leave_sequence_index();
            index = index.wrapping_add(1);
        }

        Ok(())
    })
}
