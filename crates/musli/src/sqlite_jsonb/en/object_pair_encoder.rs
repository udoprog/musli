use core::marker::PhantomData;

use crate::en::EntryEncoder;
use crate::{Context, Writer};

use super::{JsonbEncoder, JsonbObjectKeyEncoder};

/// Encoder for a single JSONB object entry.
///
/// The entries of an object are simply its keys and values written after one
/// another, so this writes straight into the payload of the enclosing object.
pub(crate) struct JsonbObjectPairEncoder<W, C, M> {
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbObjectPairEncoder<W, C, M> {
    #[inline]
    pub(super) const fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            _marker: PhantomData,
        }
    }
}

impl<W, C, M> EntryEncoder for JsonbObjectPairEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = JsonbObjectKeyEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = JsonbEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        Ok(JsonbObjectKeyEncoder::new(
            self.cx,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
