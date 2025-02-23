use crate::Context;

use super::{Encode, Encoder, SequenceEncoder};

/// The default implementation of [`Encoder::encode_slice`].
#[inline]
pub fn default_encode_slice<E, T>(encoder: E, slice: impl AsRef<[T]>) -> Result<(), E::Error>
where
    E: Encoder,
    T: Encode<E::Mode>,
{
    let cx = encoder.cx();
    let slice = slice.as_ref();
    let mut seq = encoder.encode_sequence(slice.len())?;

    for (index, item) in slice.iter().enumerate() {
        cx.enter_sequence_index(index);
        seq.push(item)?;
        cx.leave_sequence_index();
    }

    seq.finish_sequence()
}

/// The default implementation of [`Encoder::encode_slices`].
#[inline]
pub fn default_encode_slices<E, T>(
    encoder: E,
    len: usize,
    slices: impl IntoIterator<Item: AsRef<[T]>>,
) -> Result<(), E::Error>
where
    E: Encoder,
    T: Encode<E::Mode>,
{
    let cx = encoder.cx();

    let mut seq = encoder.encode_sequence(len)?;

    let mut index = 0;

    for slice in slices {
        for item in slice.as_ref() {
            cx.enter_sequence_index(index);
            seq.push(item)?;
            cx.leave_sequence_index();
            index = index.wrapping_add(1);
        }
    }

    seq.finish_sequence()
}

/// The default implementation of [`SequenceEncoder::encode_slice`].
#[inline]
pub fn default_sequence_encode_slice<E, T>(
    seq: &mut E,
    slice: impl AsRef<[T]>,
) -> Result<(), E::Error>
where
    E: ?Sized + SequenceEncoder,
    T: Encode<E::Mode>,
{
    let cx = seq.cx();

    let mut index = 0usize;

    for value in slice.as_ref() {
        cx.enter_sequence_index(index);
        seq.push(value)?;
        cx.leave_sequence_index();
        index = index.wrapping_add(1);
    }

    Ok(())
}

/// The default implementation of [`SequenceEncoder::encode_slices`].
#[inline]
pub fn default_sequence_encode_slices<E, T>(
    seq: &mut E,
    slices: impl IntoIterator<Item: AsRef<[T]>>,
) -> Result<(), E::Error>
where
    E: ?Sized + SequenceEncoder,
    T: Encode<E::Mode>,
{
    let cx = seq.cx();

    let mut index = 0usize;

    for slice in slices {
        for value in slice.as_ref() {
            cx.enter_sequence_index(index);
            seq.push(value)?;
            cx.leave_sequence_index();
            index = index.wrapping_add(1);
        }
    }

    Ok(())
}
